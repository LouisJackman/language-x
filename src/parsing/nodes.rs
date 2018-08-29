use multiphase::{Identifier, InterpolatedString, Shebang, SyDoc, SylanString};
use std::collections::{HashSet, LinkedList};
use std::hash::{Hash, Hasher};
use std::rc::Rc;
use version::Version;

// Sylan consists of items and expressions. Items are declarative whereas expressions are executed
// and yield values. Such values can be the void value for expressions executed solely for
// side-effects. Non-declaration statements don't exist but can be approximated by stacking
// expressions one after the other and discarding their values.

pub struct File {
    pub shebang: Option<Shebang>,
    pub version: Option<Version>,
    pub package: FilePackage,
}

pub enum Item {
    Package(Package),
    Class(Class),
    Interface(Interface),
    Method(Method),
    Binding(Binding),
    SyDoc(SyDoc),
}

#[derive(Clone, Eq, PartialEq)]
pub enum Expression {
    ContextualScope(ContextualScope),
    Scope(Scope),
    Function(Function),
    Identifier(Identifier),
    Literal(Literal),
    UnaryOperator(UnaryOperator, Box<Expression>),
    BinaryOperator(BinaryOperator, Box<Expression>, Box<Expression>),
    Switch(Switch),
    Select(Select),
    DoContext(Box<Expression>, Scope),
    If(If),
    For(Vec<Binding>, Scope),
    Continue(Vec<Argument<Expression>>),
    Call(Call),
    PackageLookup(PackageLookup),
    Throw(Throw),
}

pub enum Node {
    File(File),
    Item(Item),
    Expression(Expression),
}

// Packages only have declarative constructs, with the exception of the main package that can also
// have executable code to simplify small scripts.

#[derive(Clone)]
pub struct Package {
    pub accessibility: Accessibility,
    pub imports: Vec<Import>,
    pub declarations: Vec<Declaration>,
    pub name: Identifier,
}

pub struct MainPackage {
    pub package: Package,
    pub code: Code,
}

pub enum FilePackage {
    EntryPoint(MainPackage),
    Imported(Package),
}

#[derive(Clone)]
pub struct Import {
    pub lookup: PackageLookup,
}

// Declarations only have accessibility in packages and classes. In scopes, they are always public
// in that scope.

#[derive(Clone)]
pub enum Accessibility {
    Public,
    Private,
}

#[derive(Clone)]
pub enum DeclarationItem {
    Binding(Binding),
    Type(Type),
    Package(Package),
}

#[derive(Clone)]
pub struct Declaration {
    pub accessibility: Accessibility,
    pub item: DeclarationItem,
}

// There are concrete classes and interfaces, the latter being more similar to typeclasses and
// traits than traditional protocol-like OO interfaces. Concrete classes can only implement
// interfaces; only interfaces can extend other types, and those types can only be interfaces.

pub struct Class {
    pub implements: LinkedList<Type>,
    pub methods: HashSet<ConcreteMethod>,
    pub getters: HashSet<Getter>,
    pub items: HashSet<Declaration>,
}

pub struct Interface {
    pub extends: LinkedList<Type>,
    pub getters: HashSet<Getter>,
    pub methods: HashSet<Method>,
}

pub enum TypeItem {
    Class(Class),
    Interface(Interface),
}

pub struct TypeSpecification {
    pub name: Identifier,
    pub item: TypeItem,
}

pub struct NewType {
    pub type_parameters: Vec<TypeParameter>,
    pub specification: TypeSpecification,
}

pub struct TypeAssignment {
    pub name: Identifier,
    pub type_parameters: Vec<TypeParameter>,
    pub assignee: Type,
}

#[derive(Clone, Eq, PartialEq)]
pub struct Type {
    pub name: Identifier,
    pub arguments: Vec<Argument<Type>>,
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
    pub method_type: Type,
    pub function: Function,
    pub overrides: bool,
}

pub struct AbstractMethod {
    pub method_type: Type,
    pub signature: FunctionSignature,
}

pub enum MethodItem {
    Concrete(ConcreteMethod),
    Abstract(AbstractMethod),
}

pub struct Method {
    pub name: Identifier,
    pub item: MethodItem,
}

// Getters are basically just methods without the ability to specify type or value parameters. They
// have a single type, similar to a field. They are invoked without the call syntax with
// parentheses.

pub struct ConcreteGetter {
    pub body: Scope,
    pub overrides: bool,
}

pub enum GetterItem {
    Concrete(ConcreteGetter),
    Abstract,
}

pub struct Getter {
    pub getter_type: Type,
    pub name: Identifier,
    pub item: GetterItem,
}

// Type and value parameters are the same except for two differences: type parameters are for types
// at compiletime whereas value parameters are for values at runtime, and only type parameters can
// have optional upperbounds. They both have identifiers and optional default values.

#[derive(Clone, Eq, PartialEq)]
pub struct Parameter<T> {
    pub default_value: Option<T>,
    pub identifier: Identifier,
}

pub type ValueParameter = Parameter<Expression>;

#[derive(Clone, Eq, PartialEq)]
pub struct TypeParameter {
    pub parameter: Parameter<Identifier>,
    pub upper_bounds: LinkedList<Type>,
}

// Type and value arguments are the same except for one difference: type arguments are for types at
// compiletime whereas value arguments are for values at runtime. Both support being passed as
// positional or keyword arguments; unlike other languages it is the choice of the caller rather
// than the definer. If passed as a keyword argument, an identifier is carried with it in the parse
// tree.
#[derive(Clone, Eq, PartialEq)]
pub struct Argument<T> {
    pub value: T,
    pub identifier: Option<Identifier>,
}

type ValueArgument = Argument<Expression>;
type TypeArguments = Argument<Type>;

// Sylan's "symbol tables" are just a collection of bindings in the current scope. Parent scopes
// can be looked up to find bindings in outer closures, which is how lexical scoping is
//
// Bindings are for execution-time values. Statically deducable values go via package and type
// definitions instead. (Note that "execution-time" can mean both "runtime" and "running within a
// compile-time macro.)

#[derive(Clone, Eq, PartialEq)]
pub struct Binding {
    pub pattern: Pattern,
    pub value: Expression,
}

impl Hash for Binding {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.pattern.hash(state)
    }
}

#[derive(Clone, Eq, PartialEq)]
pub struct ContextualBinding {
    pub name: Identifier,
    pub value: Expression,
}

impl Hash for ContextualBinding {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state)
    }
}

// Expressions are seperate from bindings.
type Expressions = Vec<Expression>;

// Declarations within a code block are expected to be fully resolved before executing its
// expressions. This is to allow techniques like mutual recursion and self-referencial methods
// without forward declarations.
//
// Note that the declarations aren't accessible until their declarations have been executed, but
// don't cause compilation problems if accessed within a delayed computation within the same scope.
//
// In other words, these declarations are block scoped with a temporal dead zone rather than using
// scope hoisting.

#[derive(Clone, Eq, PartialEq)]
pub struct Code {
    pub bindings: HashSet<Binding>,
    pub expressions: Expressions,
}

impl Code {
    pub fn new() -> Self {
        Self {
            bindings: HashSet::new(),
            expressions: vec![],
        }
    }
}

#[derive(Clone, Eq, PartialEq)]
pub struct ContextualCode {
    pub bindings: HashSet<Binding>,
    pub contextual_bindings: HashSet<ContextualBinding>,
    pub expressions: Expressions,
}

// Scopes, unlike non-main packages, can contain executable code. Unlike all packages, they can be
// within do contexts and refer to parent scopes. They can declare variables with bindings but
// cannot declare new types or subpackages like packages can.
//
// All functions, methods, and getters have an attached scope. Scopes can also be standalone, in
// which case they are immediately invoked and then destroyed afterwards. In this case they
// function similarly to the immediately-invoked functions or do-blocks of other languages.

#[derive(Clone, Eq, PartialEq)]
pub struct Scope {
    pub code: Code,
    pub parent: Option<Rc<Scope>>,
}

impl Scope {
    pub fn new_root() -> Rc<Self> {
        let code = Code::new();
        let parent = None;
        Rc::new(Self { code, parent })
    }

    pub fn within(parent: &Rc<Scope>) -> Rc<Self> {
        let code = Code::new();
        let parent = Some(parent.clone());
        Rc::new(Self { code, parent })
    }
}

#[derive(Clone, Eq, PartialEq)]
pub struct ContextualScope {
    pub code: ContextualCode,
    pub parent: Option<Rc<Scope>>,
}

impl ContextualScope {
    pub fn new_root() -> Self {
        let code = ContextualCode {
            bindings: HashSet::new(),
            contextual_bindings: HashSet::new(),
            expressions: vec![],
        };
        let parent = None;
        Self { code, parent }
    }
}

// Like methods, functions have a scope and type and value parameters. Unlike methods, they do not
// carry references to types or instances, and cannot be overridden or be abstract.
//
// There is no difference between a function or a lambda. A lambda is merely a function that isn't
// attached to a binding in a scope. After being lexed from different tokens, they become
// indistinguishable in the AST.

#[derive(Clone, Eq, PartialEq)]
pub struct FunctionSignature {
    pub type_parameters: Vec<TypeParameter>,
    pub value_parameters: Vec<ValueParameter>,
    pub return_type: Type,
}

#[derive(Clone, Eq, PartialEq)]
pub struct Function {
    pub signature: FunctionSignature,
    pub scope: Scope,
}

#[derive(Clone, Eq, PartialEq)]
pub enum Literal {
    Boolean(bool),
    Char(char),

    // Reentering the lexer is needed for interpolations in interpolated strings.
    InterpolatedString(InterpolatedString),

    Number(i64, u64),
    String(SylanString),
}

pub type PackageLookup = Vec<Identifier>;

// Sylan allows overridding existing operators but not defining new ones,
// otherwise an operator would be an `Identifier` instead of in an enum.
//
// `=` for assignment is not an AST node in Sylan but is instead a required
// token while parsing a `Binding` node.

#[derive(Clone, Eq, PartialEq)]
pub enum UnaryOperator {
    BitwiseNot,
    BitwiseXor,
    MethodHandle,
    Negate,
    Not,
    Positive,
}

#[derive(Clone, Eq, PartialEq)]
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

#[derive(Clone, Eq, PartialEq)]
pub struct Switch {
    pub expression: Box<Expression>,
    pub cases: Vec<Case>,
}

#[derive(Clone, Eq, PartialEq)]
pub struct Timeout {
    pub nanoseconds: Box<Expression>,
    pub body: Box<Expression>,
}

#[derive(Clone, Eq, PartialEq)]
pub struct Select {
    pub cases: Vec<Case>,
    pub timeout: Option<Timeout>,
}

#[derive(Clone, Eq, PartialEq)]
pub struct Call {
    pub target: Box<Expression>,
    pub arguments: Vec<Argument<Expression>>,
}

#[derive(Clone, Eq, PartialEq)]
pub struct If {
    pub condition: Box<Expression>,
    pub then: Scope,
    pub else_clause: Option<Scope>,
}

#[derive(Clone, Eq, PartialEq)]
pub struct Case {
    pub matches: LinkedList<Pattern>,
    pub body: Expression,
}

// Throwing an expression does not yield a value as it destroys its current process. However, it is
// an expression and can therefore be used anywhere an expression can be used. It can throw any
// expression that yields a type which implements the Exception interface. In "returns" the bottom
// type, allowing it to be used everywhere.
#[derive(Clone, Eq, PartialEq)]
pub struct Throw(pub Box<Expression>);

#[derive(Clone, Eq, PartialEq)]
pub enum PatternField {
    Field(Pattern, Identifier),
    IgnoreRest,
}

#[derive(Clone, Eq, PartialEq)]
pub struct CompositePattern {
    pub composite_type: Type,
    pub components: Vec<PatternField>,
}

#[derive(Clone, Eq, PartialEq)]
pub enum PatternItem {
    Literal(Literal),
    Identifier(Identifier),
    Ignored,
    Composite(CompositePattern),
}

#[derive(Clone, Eq, PartialEq)]
pub struct Pattern {
    pub item: PatternItem,
    pub bound_match: Option<Identifier>,
}

impl Hash for Pattern {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.bound_match.hash(state)
    }
}
