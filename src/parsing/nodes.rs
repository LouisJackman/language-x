//! Sylan consists of items and expressions. Items are declarative whereas
//! expressions are executed and yield values. Such values can be the void value
//! for expressions executed solely for side-effects. Non-declaration statements
//! don't exist but can be approximated by stacking expressions one after the
//! other and discarding their values.

use std::collections::{HashSet, LinkedList};
use std::hash::{Hash, Hasher};
use std::rc::Rc;

use common::multiphase::{Identifier, InterpolatedString, Shebang, SyDoc, SylanString};
use common::version::Version;

/// Shebangs and source versions are special, which is why they're outside of
/// the `FilePackage` in which all other items and expressions reside. Both
/// shebangs must be completely resolved before anything else can be parsed,
/// and the result of parsing version can completely change the lexing and
/// parsing of all subsequent tokens.
pub struct File {
    pub shebang: Option<Shebang>,
    pub version: Option<Version>,
    pub package: FilePackage,
}

/// The declarations that make up the structure of a Sylan program.
pub enum Item {
    Package(Package),
    Class(Class),
    Interface(Interface),
    Function(Function),
    Method(Method),
    Binding(Binding),
    SyDoc(SyDoc),
}

/// The expressions that allow Turing-complete computations, i.e. allowing
/// Sylan to do actual useful work.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Expression {
    ContextualScope(ContextualScope),
    Scope(Scope),
    Lambda(Lambda),
    Identifier(Identifier),
    Literal(Literal),
    UnaryOperator(UnaryOperator, Box<Expression>),
    BinaryOperator(BinaryOperator, Box<Expression>, Box<Expression>),
    Switch(Switch),
    Select(Select),
    DoContext(Box<Expression>, Scope),
    If(If),
    For(For),
    Continue(Continue),
    Call(Call),
    PackageLookup(PackageLookup),
    Throw(Throw),
}

/// Every node in Sylan is either an item or an expression, even the special
/// shebang and version tokens (both of which are technically items).
pub enum Node {
    Item(Item),
    Expression(Expression),
}

/// Packages only have items at top-level, with the exception of the main package that can also have
/// executable code to simplify small scripts.
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

    // TODO: this needs to be removed; each package will be lexed and parsed individually
    // and linked in a later phase, probably in the simplification but more likely one of the
    // backends.
    Imported(Package),
}

#[derive(Clone)]
pub struct Import {
    pub lookup: PackageLookup,
}

#[derive(Clone)]
pub enum Accessibility {
    Public,
    Internal,
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

/// Concrete classes that support implementing interfaces and embedding other
/// classes, but cannot extend other classes directly.
pub struct Class {
    pub implements: LinkedList<Type>,
    pub methods: HashSet<ConcreteMethod>,
    pub getters: HashSet<Getter>,
    pub items: HashSet<Declaration>,
}

/// Interfaces that support extending other interfaces, providing empty methods
/// that implementors must implement, providing already-defined utility methods,
/// and even allowing already-defined methods to be specialised via overriding
/// in implementing classes.
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

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Type {
    pub name: Identifier,
    pub arguments: Vec<Argument<Type>>,
}

pub enum TypeDeclaration {
    New(NewType),
    Extension(TypeSpecification),
    Assignment(TypeAssignment),
}

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

/// Methods, which can be potentially abstract (i.e. with undefined bodies) in
/// interfaces, can be overridable and virtual in interfaces, and must be tied
/// to either a class an interface.
///
/// Otherwise they are higher-order constructs that can be passed around like
/// normal functions and lambdas. Like Python and unlike JS, their reference
/// to their type and instance are bound to the method itself.
pub struct Method {
    pub name: Identifier,
    pub item: MethodItem,
}

pub struct ConcreteGetter {
    pub body: Scope,
    pub overrides: bool,
}

pub enum GetterItem {
    Concrete(ConcreteGetter),
    Abstract,
}

/// Getters are essentially just methods without the ability to specify type or
/// value parameters. They have a single type, similar to a field. They are
/// invoked without the call syntax with parentheses.
pub struct Getter {
    pub getter_type: Type,
    pub name: Identifier,
    pub item: GetterItem,
}

/// Value parameters are for values at runtime and have identifiers and
/// optional default values.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValueParameter {
    pub pattern: Pattern,
    pub default_value: Option<Expression>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValueParameterWithTypeAnnotation {
    parameter: ValueParameter,
    type_annotation: Type,
}

/// Type parameters are for types at compile-time and have optional upper
/// bounds, identifiers, and optional default values.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TypeParameter {
    pub name: Identifier,
    pub upper_bounds: LinkedList<Type>,
    pub default_value: Option<Type>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Argument<T> {
    pub value: T,
    pub identifier: Option<Identifier>,
}

/// Value arguments are for values at runtime. They support being passed as
/// positional or keyword arguments; unlike other languages it is the choice of
/// the caller rather than the definer. If passed as a keyword argument, an
/// identifier is carried with it in the parse tree.
type ValueArgument = Argument<Expression>;

/// Type arguments are for values at runtime. They support being passed as
/// positional or keyword arguments; unlike other languages it is the choice of
/// the caller rather than the definer. If passed as a keyword argument, an
/// identifier is carried with it in the parse tree.
type TypeArguments = Argument<Type>;

// Sylan's "symbol tables" are just a collection of bindings in the current
// scope. Parent scopes can be looked up to find bindings in outer closures,
// which is how lexical scoping is implemented.

/// Bindings are for execution-time values. Statically deducible values go via
/// package and type definitions instead. (Note that "execution-time" can mean
/// both "runtime" and "running within a compile-time macro.)
#[derive(Clone, Debug, Eq)]
pub struct Binding {
    pub pattern: Pattern,
    pub value: Expression,
}

impl PartialEq for Binding {
    fn eq(&self, other: &Self) -> bool {
        self.pattern == other.pattern
    }
}

impl Hash for Binding {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.pattern.hash(state)
    }
}

#[derive(Clone, Debug, Eq)]
pub struct ContextualBinding {
    pub name: Identifier,
    pub value: Expression,
}

impl PartialEq for ContextualBinding {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Hash for ContextualBinding {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state)
    }
}

/// Expressions are seperate from bindings.
type Expressions = Vec<Expression>;

/// Bindings within a code block are resolved before executing its
/// expressions, which is why they're items rather than expressions.
/// This is to allow techniques like mutual recursion and
/// self-referential functions and methods without forward declarations.
///
/// Note that the declarations aren't accessible until their declarations have
/// been executed, but don't cause compilation problems if accessed within a
/// delayed computation within the same scope.
///
/// In other words, these declarations are block scoped with a temporal dead
/// zone rather than using scope hoisting.
#[derive(Clone, Debug, Eq, PartialEq)]
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

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContextualCode {
    pub bindings: HashSet<Binding>,
    pub contextual_bindings: HashSet<ContextualBinding>,
    pub expressions: Expressions,
}

/// Scopes, unlike non-main packages, can contain executable code. Unlike all
/// packages, they can refer to parent scopes. They can declare variables with
/// bindings but cannot declare new types or subpackages like packages can.
///
/// All functions, methods, lambdas, and getters have an attached scope.
#[derive(Clone, Debug, Eq, PartialEq)]
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

#[derive(Clone, Debug, Eq, PartialEq)]
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

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FunctionSignature {
    pub type_parameters: Vec<TypeParameter>,
    pub value_parameters: Vec<ValueParameterWithTypeAnnotation>,
    pub return_type: Type,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LambdaSignature {
    pub type_parameters: Vec<TypeParameter>,
    pub value_parameters: Vec<ValueParameter>,
}

/// Like methods, functions have a scope and type and value parameters. Unlike
/// methods, they do not carry references to types or instances, and cannot be
/// overridden or be abstract.
///
/// There is no difference between a function or a lambda. A lambda is merely a
/// function that isn't attached to a binding in a scope. After being lexed from
/// different tokens they become indistinguishable in the AST.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Function {
    pub signature: FunctionSignature,
    pub scope: Scope,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Lambda {
    pub signature: LambdaSignature,
    pub scope: Scope,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Literal {
    Boolean(bool),
    Char(char),

    /// Reentering the lexer is needed for interpolations in interpolated
    /// strings.
    ///
    /// TODO: this could lead to impenetrable Sylan code and significant lexer
    /// complications. Let's just allow interpolating symbols, nothing else,
    /// and call it a day.
    InterpolatedString(InterpolatedString),

    Number(i64, u64),
    String(SylanString),
}

pub type PackageLookup = Vec<Identifier>;

/// Sylan allows overridding existing operators but not defining new ones,
/// otherwise an operator would be an `Identifier` instead of in an enum.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum UnaryOperator {
    BitwiseNot,
    BitwiseXor,
    MethodHandle,
    Negate,
    Not,
    Positive,
}

/// Sylan allows overridding existing operators but not defining new ones,
/// otherwise an operator would be an `Identifier` instead of in an enum.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BinaryOperator {
    Add,
    And,
    Assign,
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

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Switch {
    pub expression: Box<Expression>,
    pub cases: Vec<Case>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Timeout {
    pub nanoseconds: Box<Expression>,
    pub body: Box<Expression>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Select {
    pub cases: Vec<Case>,
    pub timeout: Option<Timeout>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Call {
    pub target: Box<Expression>,
    pub arguments: Vec<Argument<Expression>>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct If {
    pub condition: Box<Expression>,
    pub then: Scope,
    pub else_clause: Option<Scope>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Case {
    pub matches: LinkedList<Pattern>,
    pub body: Expression,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct For {
    pub bindings: Vec<Binding>,
    pub scope: Scope,
    pub label: Option<Identifier>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Continue {
    pub bindings: Vec<Argument<Expression>>,
    pub label: Option<Identifier>,
}

/// Throwing an expression does not yield a value as it destroys its current
/// process. However, it is an expression and can therefore be used anywhere an
/// expression can be used. It can throw any expression that yields a type which
/// implements the Exception interface. In "returns" the bottom type which
/// allows it to be used anywhere.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Throw(pub Box<Expression>);

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PatternField {
    pub identifier: Identifier,
    pub pattern: Pattern,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompositePattern {
    pub composite_type: Type,
    pub fields: Vec<PatternField>,
    pub ignore_rest: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PatternItem {
    Literal(Literal),
    Identifier(Identifier),
    Ignored,
    Composite(CompositePattern),
}

#[derive(Clone, Debug, Eq)]
pub struct Pattern {
    pub item: PatternItem,
    pub bound_match: Option<Identifier>,
}

impl PartialEq for Pattern {
    fn eq(&self, other: &Self) -> bool {
        self.bound_match == other.bound_match
    }
}

impl Hash for Pattern {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.bound_match.hash(state)
    }
}
