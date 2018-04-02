use std::collections::{HashSet, LinkedList};
use std::sync::Arc;

// Sylan consists of items and expressions. Items are declarative whereas expressions are executed
// and yield values. Such values can be the void value for expressions executed solely for
// side-effects. Non-declaration statements don't exist but can be approximated by stacking
// expressions one after the other and discarding their values.

pub enum Item {
    Package(Package),
    Class(Class),
    Interface(Interface),
    Method(Method),
    Binding(Binding),
    SyDoc(Arc<String>),
}

pub enum Expression {
    Scope(Scope),
    Function(Function),
    Identifier(Identifier),
    Literal(Literal),
    UnaryOperator(UnaryOperator, Box<Expression>),
    BinaryOperator(BinaryOperator, Box<Expression>, Box<Expression>),
    Switch(Box<Switch>),
    Select(Select),
    DoContext(Box<Expression>, Scope),
    If(Box<If>),
    For(Vec<ForClause>, Scope),
    Continue(Vec<Argument<Expression>>),
    Call(Box<Call>),
    PackageLookup(PackageLookup),
}

pub enum Node {
    File(File),
    Item(Item),
    Expression(Expression),
}

// Packages only have declarative constructs, with the exception of the main package that can also
// have executable code to simplify small scripts.

pub struct Package {
    accessibility: Accessibility,
    imports: Vec<Import>,
    declarations: Vec<Declaration>,
    name: Arc<String>,
}

pub struct MainPackage {
    package: Package,
    code: Code,
}

pub enum FilePackage {
    EntryPoint(MainPackage),
    Imported(Package),
}

pub struct File {
    shebang: Option<Arc<String>>,
    version: Option<(u64, u64)>,
    package: FilePackage,
}

pub struct Import {
    lookup: PackageLookup,
}

// Declarations only have accessibility in packages and classes. In scopes, they are always public
// in that scope.

pub enum Accessibility {
    Public,
    Private,
}

pub enum DeclarationItem {
    Binding(Binding),
    Type(Type),
    Package(Package),
}

pub struct Declaration {
    accessibility: Accessibility,
    item: DeclarationItem,
}

// There are concrete classes and interfaces, the latter being more similar to typeclasses and
// traits than traditional protocol-like OO interfaces. Concrete classes can only implement
// interfaces; only interfaces can extend other types, and those types can only be interfaces.

pub struct Class {
    implements: LinkedList<Interface>,
    methods: HashSet<ConcreteMethod>,
    getters: HashSet<Getter>,
    items: HashSet<Declaration>,
}

pub struct Interface {
    extends: LinkedList<Interface>,
    getters: HashSet<Getter>,
    methods: HashSet<Method>,
}

pub enum TypeItem {
    Class(Class),
    Interface(Interface),
}

pub struct TypeSpecification {
    name: Identifier,
    item: TypeItem,
}

pub struct NewType {
    type_parameters: Vec<TypeParameter>,
    specification: TypeSpecification,
}

pub struct TypeAssignment {
    name: Identifier,
    type_parameters: Vec<TypeParameter>,
    assignee: Type,
}

pub struct Type {
    name: Identifier,
    arguments: Vec<Argument<Type>>,
}

pub enum TypeDeclaration {
    New(NewType),
    Extension(TypeSpecification),
    Assignment(TypeAssignment),
}

// Methods and functions are different constructs; only methods can be overridden, be abstract, and
// must be tied to a type. Otherwise they are higher-order constructs that can be passed around
// like normal functions. Like Python and unlike JS, their reference to their type and instance are
// bound to the method itself.

pub struct ConcreteMethod {
    method_type: Type,
    function: Function,
    overrides: bool,
}

pub struct AbstractMethod {
    method_type: Type,
    signature: FunctionSignature,
}

pub enum MethodItem {
    Concrete(ConcreteMethod),
    Abstract(AbstractMethod),
}

pub struct Method {
    name: Identifier,
    item: MethodItem,
}

// Getters are basically just methods without the ability to specify type or value parameters. They
// have a single type, similar to a field. They are invoked without the call syntax with
// parentheses.

pub struct ConcreteGetter {
    body: Scope,
    overrides: bool,
}

pub enum GetterItem {
    Concrete(ConcreteGetter),
    Abstract,
}

pub struct Getter {
    getter_type: Type,
    name: Identifier,
    item: GetterItem,
}

// Type and value parameters are the same except for two differences: type parameters are for types
// at compiletime whereas value parameters are for values at runtime, and only type parameters can
// have optional upperbounds. They both have identifiers and optional default values.

pub struct Parameter<T> {
    default_value: Option<T>,
    identifier: Identifier,
}

pub struct TypeParameter {
    parameter: Parameter<Identifier>,
    upper_bound: Option<Type>,
}

type ValueParameter = Parameter<Type>;

// Type and value arguments are the same except for one difference: type arguments are for types at
// compiletime whereas value arguments are for values at runtime. Both support being passed as
// positional or keyword arguments; unlike other languages it is the choice of the caller rather
// than the definer. If passed as a keyword argument, an identifier is carried with it in the parse
// tree.
pub struct Argument<T> {
    value: T,
    identifier: Option<Identifier>,
}

// Sylan's "symbol tables" are just a collection of bindings in the current scope. Parent scopes
// can be looked up to find bindings in outer closures, which is how lexical scoping is
// implemented.

pub enum Identifier {
    Actual(Arc<String>),
    Ignored,
}

pub struct Binding {
    binding_type: Type,
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
pub struct Scope {
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

pub struct FunctionSignature {
    type_parameters: Vec<TypeParameter>,
    value_parameters: Vec<ValueParameter>,
    return_type: Type,
}

pub struct Function {
    signature: FunctionSignature,
    scope: Scope,
}

pub enum Literal {
    Boolean(bool),
    Char(char),

    // Reentering the lexer is needed for interpolations in interpolated strings.
    InterpolatedString(Arc<String>),

    Number(i64, u64),
    String(Arc<String>),
}

type PackageLookup = Vec<Identifier>;

// Sylan allows overridding existing operators but not defining new ones,
// otherwise an operator would be an `Identifier` instead of in an enum.
//
// `=` for assignment is not an operator in Sylan but is instead a required
// token while parsing a `Binding` node.

pub enum UnaryOperator {
    BitwiseNot,
    BitwiseXor,
    MethodHandle,
    Negate,
    Not,
    Positive,
}

pub enum BinaryOperator {
    Add,
    And,
    BitwiseAnd,
    BitwiseOr,
    Compose,
    Divide,
    Dot,
    Equals,
    GreaterThan,
    GreaterThanOrEquals,
    LessThan,
    LessThenOrEquals,
    Modulo,
    Multiply,
    NotEquals,
    Or,
    Pipe,
    ShiftLeft,
    ShiftRight,
    Subtract,
}

pub struct Switch {
    expression: Expression,
    cases: Vec<SwitchCase>,
    default: Scope,
}

pub struct Timeout {
    nanoseconds: usize,
    body: Scope,
}

pub struct Select {
    cases: Vec<SelectCase>,
    timeout: Timeout,
}

pub struct Call {
    target: Expression,
    arguments: Vec<Argument<Expression>>,
}

pub struct If {
    condition: Expression,
    then: Scope,
    else_clause: Option<Scope>,
}

pub struct SwitchCase {
    matches: LinkedList<Expression>,
    body: Scope,
}

pub struct SelectCase {
    matches: LinkedList<Pattern>,
    body: Scope,
}

pub struct ForClause {
    identifier: Identifier,
    initial_value: Expression,
}

// Throwing an expression does not yield a value as it destroys its current process. However, it is
// an expression and can therefore be used anywhere an expression can be used. It can throw any
// expression that yields a type which implements the Exception interface.
pub struct Throw(Expression);

pub enum PatternField {
    Identifier(Identifier),
    BoundIdentifier(Identifier, Pattern),
    IgnoreRest,
}

pub struct CompositePattern {
    composite_type: Type,
    components: Vec<PatternField>,
}

pub enum PatternItem {
    Literal,
    Identifier,
    Expression(Expression),
    Composite(CompositePattern),
}

pub struct Pattern {
    item: PatternItem,
    binding: Option<Identifier>,
}
