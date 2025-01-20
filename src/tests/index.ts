// Keyword types
let anyValue: any;
let bigIntValue: bigint = 100n;
let booleanValue: boolean = true;
let neverValue: never;
let nullValue: null = null;
let numberValue: number = 42;
let objectValue: object = {};
let stringValue: string = "Hello";
let symbolValue: symbol = Symbol("unique");
let undefinedValue: undefined = undefined;
let unknownValue: unknown;
let voidValue: void;

// Interface
interface Person {
  name: string;
  age: number;
}

// Type Alias
type myString = string;

// Literal
const literal = "example";
const number_literal = 42;
const boolean_literal = true;
const object_literal = {};
const null_literal = null;
const undefined_literal = undefined;

// Function
function add(a: number, b: number): number {
  return a + b;
}

// Reference
type UserID = string;
let userId: UserID = "user123";
