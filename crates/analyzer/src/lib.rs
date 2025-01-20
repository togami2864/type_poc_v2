use biome_js_syntax::{
    AnyJsExpression, AnyJsFormalParameter, AnyJsFunction, AnyJsLiteralExpression, AnyJsModuleItem,
    AnyJsParameter, AnyJsRoot, AnyJsStatement, AnyTsName, AnyTsReturnType, AnyTsType,
    AnyTsTypeMember, AnyTsVariableAnnotation, JsExpressionStatement, JsFunctionDeclaration,
    JsFunctionExpression, JsModule, JsParameters, JsSyntaxNode, JsVariableStatement,
    TsInterfaceDeclaration, TsTypeAliasDeclaration, TsTypeAnnotation,
};
use biome_rowan::{AstNode, SyntaxError};
use rustc_hash::FxHashMap;
use std::path::PathBuf;
use type_info::type_info::{TsKeywordType, TypeInfo};
use visitor::AstVisitor;

#[derive(Default)]
struct SymbolTable {
    symbols: FxHashMap<String, TypeInfo>,
}

impl SymbolTable {
    fn new() -> Self {
        Self {
            symbols: FxHashMap::default(),
        }
    }

    fn insert(&mut self, name: String, type_info: TypeInfo) {
        self.symbols.insert(name, type_info);
    }

    fn get(&self, name: &str) -> Option<&TypeInfo> {
        self.symbols.get(name)
    }
}

#[derive(Default)]
pub struct Analyzer {
    current_path: PathBuf,
    type_info_table: FxHashMap<PathBuf, FxHashMap<JsSyntaxNode, TypeInfo>>,
    global_symbols: SymbolTable,
    local_symbols: FxHashMap<PathBuf, SymbolTable>,
}

impl Analyzer {
    pub fn new() -> Self {
        Self {
            current_path: PathBuf::new(),
            type_info_table: FxHashMap::default(),
            global_symbols: SymbolTable::new(),
            local_symbols: FxHashMap::default(),
        }
    }

    pub fn type_info_table(&self) -> &FxHashMap<PathBuf, FxHashMap<JsSyntaxNode, TypeInfo>> {
        &self.type_info_table
    }

    pub fn get_type_info(&self, path: &PathBuf, node: &JsSyntaxNode) -> Option<TypeInfo> {
        self.type_info_table
            .get(path)
            .and_then(|file_cache| file_cache.get(node))
            .cloned()
    }

    pub fn set_current_path(&mut self, path: PathBuf) {
        self.current_path = path;
    }
}

impl AstVisitor for Analyzer {
    fn visit(&mut self, ast: &AnyJsRoot) {
        self.visit_node(ast);
    }
}

type VResult<T> = std::result::Result<T, SyntaxError>;

impl Analyzer {
    fn visit_node(&mut self, node: &AnyJsRoot) {
        match node {
            AnyJsRoot::JsModule(node) => {
                self.visit_module(node);
            }
            _ => todo!(),
        }
    }

    fn visit_module(&mut self, node: &JsModule) {
        for item in node.items() {
            match item {
                AnyJsModuleItem::AnyJsStatement(node) => {
                    self.visit_statement(&node);
                }
                _ => todo!(),
            }
        }
    }

    fn visit_statement(&mut self, node: &AnyJsStatement) {
        match node {
            AnyJsStatement::JsExpressionStatement(node) => {
                self.visit_expression_statement(node);
            }
            AnyJsStatement::JsVariableStatement(node) => {
                self.visit_variable_statement(node);
            }
            AnyJsStatement::TsInterfaceDeclaration(node) => {
                self.visit_interface_declaration(node);
            }
            AnyJsStatement::TsTypeAliasDeclaration(node) => {
                self.visit_type_alias_declaration(node);
            }
            AnyJsStatement::JsFunctionDeclaration(node) => {
                self.visit_function_declaration(node);
            }
            _ => todo!("{}", node),
        }
    }

    fn visit_variable_statement(&mut self, node: &JsVariableStatement) -> VResult<()> {
        if let Ok(dec) = node.declaration() {
            let declarators = dec.declarators();
            for d in declarators {
                let d = d.unwrap();
                let id = d.id().unwrap();

                if let Some(type_ann) = d.variable_annotation() {
                    let ty = self.visit_variable_annotation(&type_ann)?;
                    self.type_info_table
                        .entry(self.current_path.clone())
                        .or_default()
                        .insert(id.clone().into(), ty);
                } else if let Some(init) = d.initializer() {
                    let expr = init.expression()?;
                    self.visit_expression(&expr);
                } else {
                    self.type_info_table
                        .entry(self.current_path.clone())
                        .or_default()
                        .insert(id.clone().into(), TypeInfo::Unknown);
                }
            }
        }
        Ok(())
    }

    fn visit_variable_annotation(&mut self, node: &AnyTsVariableAnnotation) -> VResult<TypeInfo> {
        match node {
            AnyTsVariableAnnotation::TsTypeAnnotation(ty) => {
                let ty = self.analyze_type_annotation(&ty.ty()?)?;
                Ok(ty)
            }
            _ => todo!("{}", node),
        }
    }

    fn visit_interface_declaration(&mut self, node: &TsInterfaceDeclaration) -> VResult<()> {
        let id = node.id()?;
        let members = node.members();
        let mut mem = Vec::new();
        for member in members {
            match member {
                AnyTsTypeMember::TsPropertySignatureTypeMember(node) => {
                    let name = node.name()?.text();
                    let ty = node.type_annotation();
                    match ty {
                        Some(ty) => {
                            let ty = self.analyze_type_annotation(&ty.ty()?)?;
                            mem.push((name, ty));
                        }
                        None => mem.push((name, TypeInfo::Unknown)),
                    }
                }
                _ => todo!("{}", member),
            }
        }
        self.type_info_table
            .entry(self.current_path.clone())
            .or_default()
            .insert(
                id.clone().into(),
                TypeInfo::Interface {
                    name: id.text(),
                    properties: mem,
                },
            );
        Ok(())
    }

    fn visit_type_alias_declaration(&mut self, node: &TsTypeAliasDeclaration) -> VResult<()> {
        let id = node.binding_identifier()?;
        let ty = node.ty()?;
        let ty = self.analyze_type_annotation(&ty)?;
        self.type_info_table
            .entry(self.current_path.clone())
            .or_default()
            .insert(
                id.clone().into(),
                TypeInfo::TypeAlias {
                    name: id.text(),
                    aliased_type: Box::new(ty),
                },
            );
        Ok(())
    }

    fn visit_function_declaration(&mut self, node: &JsFunctionDeclaration) -> VResult<()> {
        let id = node.id().unwrap();

        let params = node.parameters()?;

        let param_types = self.analyze_params(&params)?;

        let ret = node.return_type_annotation().unwrap().ty()?;
        let ret_ty = self.analyze_return_type_annotation(&ret)?;
        let function_type = TypeInfo::Function {
            params: param_types,
            return_type: Box::new(ret_ty),
        };

        self.type_info_table
            .entry(self.current_path.clone())
            .or_default()
            .insert(id.clone().into(), function_type.clone());

        self.local_symbols
            .entry(self.current_path.clone())
            .or_default()
            .insert(id.text(), function_type);

        Ok(())
    }

    fn analyze_return_type_annotation(&mut self, node: &AnyTsReturnType) -> VResult<TypeInfo> {
        match node {
            AnyTsReturnType::AnyTsType(node) => {
                let ty = self.analyze_type_annotation(&node)?;
                Ok(ty)
            }
            _ => todo!("{}", node),
        }
    }

    fn analyze_params(&mut self, node: &JsParameters) -> VResult<Vec<(String, TypeInfo)>> {
        let mut res = Vec::new();
        for param in node.items() {
            let id = param?;
            match id {
                AnyJsParameter::AnyJsFormalParameter(node) => match node {
                    AnyJsFormalParameter::JsFormalParameter(node) => {
                        let id = node.binding()?;
                        match node.type_annotation() {
                            Some(ann) => {
                                let ty = self.analyze_type_annotation(&ann.ty()?)?;
                                res.push((id.text(), ty));
                            }
                            None => todo!(),
                        };
                    }
                    _ => todo!(),
                },
                _ => todo!("{}", id),
            }
        }
        Ok(res)
    }

    fn analyze_type_annotation(&mut self, node: &AnyTsType) -> VResult<TypeInfo> {
        let ty = match node {
            AnyTsType::TsAnyType(_) => TypeInfo::Keyword(TsKeywordType::TSAnyKeyword),
            AnyTsType::TsNumberType(_) => TypeInfo::Keyword(TsKeywordType::TSNumberKeyword),
            AnyTsType::TsBooleanType(_) => TypeInfo::Keyword(TsKeywordType::TSBooleanKeyword),
            AnyTsType::TsUnknownType(_) => TypeInfo::Keyword(TsKeywordType::TSUnknownKeyword),
            AnyTsType::TsVoidType(_) => TypeInfo::Keyword(TsKeywordType::TSVoidKeyword),
            AnyTsType::TsUndefinedType(_) => TypeInfo::Keyword(TsKeywordType::TSUndefinedKeyword),
            AnyTsType::TsNullLiteralType(_) => TypeInfo::Keyword(TsKeywordType::TSNullKeyword),
            AnyTsType::TsNeverType(_) => TypeInfo::Keyword(TsKeywordType::TSNeverKeyword),
            AnyTsType::TsBigintType(_) => TypeInfo::Keyword(TsKeywordType::TSBigIntKeyword),
            AnyTsType::TsStringType(_) => TypeInfo::Keyword(TsKeywordType::TSStringKeyword),
            AnyTsType::TsSymbolType(_) => TypeInfo::Keyword(TsKeywordType::TSSymbolKeyword),
            AnyTsType::TsNonPrimitiveType(_) => TypeInfo::Keyword(TsKeywordType::TSObjectKeyword),

            // Literal
            AnyTsType::TsBigintLiteralType(node) => TypeInfo::Literal { value: node.text() },
            AnyTsType::TsNumberLiteralType(node) => TypeInfo::Literal { value: node.text() },
            AnyTsType::TsBooleanLiteralType(node) => TypeInfo::Literal { value: node.text() },
            AnyTsType::TsStringLiteralType(node) => TypeInfo::Literal { value: node.text() },

            AnyTsType::TsReferenceType(node) => {
                let id = node.name()?;
                if let Some(referenced_ty) = self.global_symbols.get(&id.text()) {
                    referenced_ty.clone()
                } else {
                    TypeInfo::Reference(id.text())
                }
            }

            AnyTsType::TsFunctionType(node) => todo!(),
            AnyTsType::TsObjectType(ts_object_type) => todo!(),
            AnyTsType::TsTemplateLiteralType(ts_template_literal_type) => todo!(),
            AnyTsType::TsUnionType(ts_union_type) => todo!(),

            _ => todo!("Unsupported type annotation: {:?}", node),
        };
        Ok(ty)
    }

    fn visit_expression_statement(&mut self, node: &JsExpressionStatement) {
        let expr = node.expression().unwrap();
        self.visit_expression(&expr);
    }

    fn visit_expression(&mut self, node: &AnyJsExpression) -> VResult<()> {
        match node {
            AnyJsExpression::AnyJsLiteralExpression(node) => match node {
                AnyJsLiteralExpression::JsNumberLiteralExpression(node) => {
                    let value = node.syntax().text().to_string();
                    self.type_info_table
                        .entry(self.current_path.clone())
                        .or_default()
                        .insert(node.clone().into(), TypeInfo::Literal { value });
                }
                AnyJsLiteralExpression::JsStringLiteralExpression(node) => {
                    let value = node.syntax().text().to_string();
                    self.type_info_table
                        .entry(self.current_path.clone())
                        .or_default()
                        .insert(node.clone().into(), TypeInfo::Literal { value });
                }
                AnyJsLiteralExpression::JsBooleanLiteralExpression(node) => {
                    let value = node.syntax().text().to_string();
                    self.type_info_table
                        .entry(self.current_path.clone())
                        .or_default()
                        .insert(node.clone().into(), TypeInfo::Literal { value });
                }
                AnyJsLiteralExpression::JsNullLiteralExpression(node) => {
                    let value = node.syntax().text().to_string();
                    self.type_info_table
                        .entry(self.current_path.clone())
                        .or_default()
                        .insert(node.clone().into(), TypeInfo::Literal { value });
                }
                AnyJsLiteralExpression::JsBigintLiteralExpression(node) => {
                    let value = node.syntax().text().to_string();
                    self.type_info_table
                        .entry(self.current_path.clone())
                        .or_default()
                        .insert(node.clone().into(), TypeInfo::Literal { value });
                }
                _ => todo!("{}", node),
            },
            AnyJsExpression::JsObjectExpression(node) => {
                todo!();
            }
            AnyJsExpression::JsIdentifierExpression(node) => {
                let id = node.name()?;
                self.type_info_table
                    .entry(self.current_path.clone())
                    .or_default()
                    .insert(node.clone().into(), TypeInfo::Reference(id.text()));
            }
            _ => todo!("{}", node),
        };
        Ok(())
    }
}
