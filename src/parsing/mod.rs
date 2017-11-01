use std::collections::{HashMap, HashSet, LinkedList};

// Sylan consists of items and expressions. Items are declarative, whereas expressions are executed
// and yield values. Such values can be the void value, for expressions executed solely for
// side-effects. Non-declaration statements don't exist but can be approximated by stacking
// expressions one after the other and discarding their values.

enum Item {
    Package(Package),
    Class(Class),
    Interface(Interface),
    Method(Method),
    Binding(Binding),
    Shebang(String),
    SyDoc(String),
    Version(f64),
}

enum Expression {
    Scope(Scope),
    Function(Function),
    Identifier(Identifier),
    Literal,
    Operator(Identifier, Box<Expression>, Box<Expression>),
    Switch(Box<Switch>),
    Select(Select),
    DoContext(Box<Expression>, Scope),
    If(Box<If>),
    For(Vec<ForClause>, Scope),
    Call(Box<Call>),
    PackageLookup(PackageLookup),
}

// Packages only have declarative constructs, with the exception of the main package that can also
// have executable code to simplify small scripts.

struct Package {
    accessibility: Accessibility,
    imports: Vec<Import>,
    declarations: Vec<Declaration>,
    name: String,
}

struct MainPackage {
    package: Package,
    code: Code,
}

struct Import {
    lookup: PackageLookup,
}

// Declarations only have accessibility in packages and classes. In scopes, they are always public
// in that scope.

enum Accessibility {
    Public,
    Internal,
    Private,
}

enum DeclarationItem {
    Binding(Binding),
    Type(Type),
    Package(Package),
}

struct Declaration {
    accessibility: Accessibility,
    item: DeclarationItem,
}

// There are concrete classes and interfaces, the latter being more similar to typeclasses and
// traits than traditional protocol-like OO interfaces. Concrete classes can only implement
// interfaces; only interfaces can extend other types, and those types can only be interfaces.

struct Class {
    implements: LinkedList<Interface>,
    methods: HashSet<ConcreteMethod>,
    getters: HashSet<Getter>,
    items: HashSet<Declaration>,
}

struct Interface {
    extends: LinkedList<Interface>,
    getters: HashSet<Getter>,
    methods: HashSet<Method>,
}

enum TypeItem {
    Class(Class),
    Interface(Interface),
}

struct TypeSpecification {
    name: Identifier,
    item: TypeItem,
}

struct NewType {
    type_parameters: Vec<TypeParameter>,
    specification: TypeSpecification,
}

struct TypeAssignment {
    name: Identifier,
    type_parameters: Vec<TypeParameter>,
    assignee: Type,
}

struct Type {
    name: Identifier,
    arguments: Vec<Argument<Type>>,
}

enum TypeDeclaration {
    New(NewType),
    Extension(TypeSpecification),
    Assignment(TypeAssignment),
}

// Methods and functions are different constructs; only methods can be overridden, be abstract, and
// must be tied to a type. Otherwise they are higher-order constructs that can be passed around
// like normal functions. Like Python and unlike JS, their reference to their type and instance
// are bound to the method itself.

struct ConcreteMethod {
    method_type: Type,
    function: Function,
    overrides: bool,
}

struct AbstractMethod {
    method_type: Type,
    signature: FunctionSignature
}

enum MethodItem {
    Concrete(ConcreteMethod),
    Abstract(AbstractMethod),
}

struct Method {
    name: Identifier,
    item: MethodItem,
}

// Getters are basically just methods without the ability to specify type or value parameters. They
// have a single type, similar to a field. They are invoked without the call syntax with
// parentheses.

struct ConcreteGetter {
    body: Scope,
    overrides: bool,
}

enum GetterItem {
    Concrete(ConcreteGetter),
    Abstract,
}

struct Getter {
    getter_type: Type,
    name: Identifier,
    item: GetterItem,
}

// Type and value parameters are the same except for two differences: type parameters are for types
// at compiletime whereas value parameters are for values at runtime, and only type parameters can
// have optional upperbounds. They both have identifiers and optional default values.

struct Parameter<T> {
    default_value: Option<T>,
    identifier: Identifier,
}

struct TypeParameter {
    parameter: Parameter<Identifier>,
    upper_bound: Option<Type>,
}

type ValueParameter = Parameter<Type>;

// Type and value arguments are the same except for one difference: type arguments are for types at
// compiletime whereas value arguments are for values at runtime. Both support being passed as
// positional or keyword arguments; unlike other languages it is the choice of the caller rather
// than the definer. If passed as a keyword argument, an identifier is carried with it in the parse
// tree.
struct Argument<T> {
    value: T,
    identifier: Option<Identifier>,
}

// Sylan's "symbol tables" are just a collection of bindings in the current scope. Parent scopes
// can be looked up to find bindings in outer closures, which is how lexical scoping is
// implemented.

enum Identifier {
    Actual(String),
    Ignored,
}

struct Binding {
    bindingType: Type,
    name: Identifier,
    value: Expression,
}

// Executable code is completely seperated from declarations. Declarations in a scope are expected
// to be fully resolved before executing its "code", which is just a sequence of expressions.
type Code = Vec<Expression>;

// Scopes, unlike non-main packages, can contain executable code. Unlike all packages, they can be
// within do contexts. Like packages, they can contain imports, new packages, and new classes. This
// allows new types and packages to be defined in functions or block scopes.
//
// All functions, methods, and getters have an attached scope. Scopes can also be standalone, in
// which case they are immediately invoked and then destroyed afterwards. In this case they
// function similarly to the immediately-invoked functions or do-blocks of other languages.
struct Scope {
    imports: Vec<Import>,
    packages: HashSet<Package>,
    type_declarations: HashSet<TypeDeclaration>,
    bindings: HashSet<Binding>,
    code: Code,
    parent: Option<Box<Scope>>,
    in_context: bool,
}

// Like methods, functions have a scope and type and value parameters. Unlike methods, they do not
// carry references to types or instances, and cannot be overridden or be abstract.
//
// There is no difference between a function or a lambda. A lambda is merely a function that isn't
// attached to a binding in a scope. After being lexed from different tokens, they become
// indistinguishable in the AST.

struct FunctionSignature {
    type_parameters: Vec<TypeParameter>,
    value_parameters: Vec<ValueParameter>,
    return_type: Type,
}

struct Function {
    signature: FunctionSignature,
    scope: Scope,
}

enum Literal {
    Boolean(bool),
    Char(char),

    // Reentering the lexer is needed for interpolations in interpolated strings.
    InterpolatedString(String),

    Number(f64),
    String(String),
}

type PackageLookup = Vec<Identifier>;

struct Switch {
    expression: Expression,
    cases: Vec<SwitchCase>,
    default: Scope,
}

struct Timeout {
    nanoseconds: usize,
    body: Scope,
}

struct Select {
    cases: Vec<SelectCase>,
    timeout: Timeout,
}

struct Call {
    target: Expression,
    arguments: Vec<Argument<Expression>>
}

struct If {
    condition: Expression,
    then: Scope,
    else_clause: Option<Scope>,
}

struct SwitchCase {
    matches: LinkedList<Expression>,
    body: Scope,
}

// TODO: this is not a real pattern; implement it properly.
struct Pattern {
    patternType: Type,
    binding: Option<Identifier>,
}

struct SelectCase {
    matches: LinkedList<Pattern>,
    body: Scope,
}

struct ForClause {
    identifier: Identifier,
    initialValue: Expression,
}

// Throwing an expression does not yield a value as it destroys its current process. However, it is
// an expression and can therefore be used anywhere an expression can be used. It can throw any
// expression that yields a type which implements the Exception interface.
struct Throw(Expression);

pub struct Parser {
    top_level: Package,
}
