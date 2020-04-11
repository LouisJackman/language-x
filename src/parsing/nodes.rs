//! Sylan consists of items and expressions. Items are declarative whereas
//! expressions are executed and yield values. Such values can be the void value
//! for expressions executed solely for side-effects. "Statements" can be approximated by stacking
//! expressions one after the other and discarding their values.

use std::collections::{HashSet, LinkedList};
use std::hash::{Hash, Hasher};
use std::rc::Rc;

use crate::common::multiphase::{
    Accessibility, Identifier, InterpolatedString, OverloadableInfixOperator, PostfixOperator,
    Shebang, SyDoc, SylanString,
};
use crate::common::version::Version;

/// Shebangs and source versions are special, which is why they're outside of
/// the `PackageFile` in which all other items and expressions reside. Both
/// shebangs must be completely resolved before anything else can be parsed,
/// and the result of parsing version can completely change the lexing and
/// parsing of all subsequent tokens.
pub struct File {
    pub shebang: Option<Shebang>,
    pub version: Option<Version>,
    pub package: PackageFile,
}

/// Every node in Sylan is either an item or an expression, even the special
/// shebang and version tokens (both of which are items).
pub enum Node {
    Item(Item),
    Expression(Expression),
}

/// The declarations that make up the static structure of a Sylan program. Items
/// can't be contained within expressions, with the exception of bindings.
pub enum Item {
    Extension(Extension),
    Fun(Fun),
    Import(SymbolLookup),
    Package(Package),
    SyDoc(SyDoc),
    Type(Type),

    // Unlike the previous variants, these can be arbitrarily nested within
    // expressions. This is to allow corecursion among other features.
    //
    // For loops also create bindings, but are not items because I can't
    // think of a use case for mutually recursive loop continuation bindings.
    Binding(Binding),
}

/// The expressions that allow Turing-complete computations, i.e. allowing
/// Sylan to do actual useful work.
pub enum Expression {
    BranchingAndJumping(BranchingAndJumping),
    Context(Scope),
    Literal(Literal),
    Operator(Operator),
    SymbolLookup(SymbolLookup),
    Throw(Throw),
    Using(Using),
}

pub enum Operator {
    InfixOperator(OverloadableInfixOperator, Box<Expression>, Box<Expression>),
    PostfixOperator(PostfixOperator, Box<Expression>),
}

pub enum BranchingAndJumping {
    Call(Call),
    Cond(Cond),
    For(For),
    If(If),
    IfLet(IfLet),
    Select(Select),
    Switch(Switch),
    While(While),
    WhileLet(WhileLet),
}

/// Packages only have items at top-level, with the exception of the main package that can also have
/// executable code to simplify small scripts.
pub struct Package {
    pub accessibility: Accessibility,
    pub name: Identifier,
    pub items: Vec<Item>,
}

pub struct MainPackage {
    pub package: Package,
    pub code: Code,
}

pub enum PackageFile {
    EntryPoint(MainPackage),
    Dependency(Package),
}

pub struct FunModifiers {
    pub accessibility: Accessibility,
    pub is_ignorable: bool,
    pub is_extern: bool,
    pub is_operator: bool,
}

// Funs are ultimately just lambdas used in a binding, the syntactic equivalent of combining `var`
// with a lambda expression. However, this is only realised during the simplification stage; at
// this stage they are still distinct.
//
// One notable difference is that omitting a return type on a lambda triggers type inference,
// whereas it always means the `Void` type for `func`. Also, `Fun` expects the lambda signature to
// explicitly type every parameter, and will quickly fail in a later stage if any type annotations
// are missing. This is because `func` is intended to be used for top-level functions that define
// the shape of the program or API, in which types should always be explicitly annotated anyway.
pub struct Fun {
    pub name: Identifier,
    pub modifiers: FunModifiers,
    pub lambda: Lambda,
}

pub enum DeclarationItem {
    Binding(Binding),
    Type(Type),
    Package(Package),
}

pub struct Declaration {
    pub accessibility: Accessibility,
    pub item: DeclarationItem,
}

pub struct ClassModifiers {
    accessibility: Accessibility,
    is_extern: bool,
}

// Concrete classes that support implementing interfaces and embedding other
// classes, but cannot extend other classes directly.

pub struct Class {
    pub implements: Vec<Type>,
    pub methods: HashSet<Method>,
    pub fields: HashSet<Field>,

    // Initialisation
    pub value_parameters: Vec<ValueParameter>,
    pub code: Code,
}

pub struct Enum {
    pub implements: Vec<Type>,
    pub methods: HashSet<Method>,
    pub fields: HashSet<Field>,

    // Initialisation
    pub value_parameters: Vec<ValueParameter>,
    pub code: Code,
}

/// Interfaces that support extending other interfaces, providing empty methods
/// that implementors must implement, providing already-defined utility methods,
/// and even allowing already-defined methods to be specialised via overriding
/// in implementing classes.
pub struct Interface {
    pub extends: Vec<Type>,
    pub methods: HashSet<Method>,
}

pub enum TypeItem {
    Class(Class),
    Enum(Enum),
    Interface(Interface),
}

pub struct Type {
    pub name: Identifier,
    pub type_parameters: Vec<TypeParameter>,
    pub item: TypeItem,
}

pub struct TypeSymbolLookup {
    pub lookup: SymbolLookup,
    pub type_arguments: Vec<TypeArgument>,
}

impl TypeSymbolLookup {
    pub fn new(lookup: Vec<Identifier>) -> TypeSymbolLookup {
        TypeSymbolLookup {
            lookup: SymbolLookup(lookup),
            type_arguments: vec![],
        }
    }
}

pub struct Extension(pub Type);

pub struct MethodModifiers {
    fun_modifiers: FunModifiers,
    overrides: bool,
    has_a_default: bool,
}

/// Methods and just bindings in a class, which can be potentially abstract (i.e. with no initial
/// value) in interfaces, can be overridable in interfaces, and must be tied to either
/// a class an interface. There is no meaningful distintion between a method and an attribute: a
/// `method` is just a binding in a class and it works like a traditional OOP method
/// when that binding contains a lambda.
///
/// They are just normal bindings but tied to their type. Like Python and unlike JS, their
/// reference to their type and instance are bound to the method itself.
///
/// Type annotations are only optional in a special case: direct literals. This means a very common
/// case, OOP-style methods, don't require spelling out type annotations twice as lambdas are
/// literals.
pub struct Method {
    pub name: Identifier,
    pub modifiers: MethodModifiers,
    pub type_annotation: Option<Type>,
    pub non_abstract_value: Option<Expression>,
}

/// Value parameters are for values at runtime and have identifiers and
/// optional default values.
pub struct ValueParameter {
    pub pattern: Pattern,
    pub explicit_type_annotation: Option<TypeSymbolLookup>,
    pub default_value: Option<Expression>,
}

/// Type parameters are for types at compile-time and have optional upper
/// bounds, identifiers, and optional default values.
pub struct TypeParameter {
    pub name: Identifier,
    pub upper_bounds: Vec<TypeSymbolLookup>,
    pub default_value: Option<TypeSymbolLookup>,
}

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
type TypeArgument = Argument<Type>;

// Sylan's "symbol tables" are just a collection of bindings in the current
// scope. Parent scopes can be looked up to find bindings in outer closures,
// which is how lexical scoping is implemented.

/// Bindings are for execution-time values. Statically deducible values go via
/// package and type definitions instead. (Note that "execution-time" can mean
/// both "runtime" and "running within a compile-time macro.)
///
/// Local bindings do not have item declaration modifiers like access modifiers,
/// whereas non-local bindings do. Non-local bindings are just bindings at the
/// package level. Bindings in classes are called "fields" which are the same
/// except they allow embedding of their values into the outer class.

pub struct LocalBinding {
    pub pattern: Pattern,
    pub value: Box<Expression>,
    pub explicit_type_annotation: Option<TypeSymbolLookup>,
}

pub struct Binding {
    pub is_extern: bool,
    pub accessibility: Accessibility,
    pub binding: LocalBinding,
}

pub struct Field {
    pub is_extern: bool,
    pub is_embedded: bool,
    pub accessibility: Accessibility,
    pub binding: LocalBinding,
}

impl PartialEq for LocalBinding {
    fn eq(&self, other: &Self) -> bool {
        self.pattern == other.pattern
    }
}

impl Hash for LocalBinding {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.pattern.hash(state)
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
pub struct Code {
    pub bindings: Vec<LocalBinding>,
    pub expressions: Expressions,
}

impl Code {
    pub fn new() -> Self {
        Self {
            bindings: vec![],
            expressions: vec![],
        }
    }
}

/// Scopes, unlike non-main packages, can contain executable code. Unlike all
/// packages, they can refer to parent scopes. They can declare variables with
/// bindings but cannot declare new types or subpackages like packages can.
///
/// All functions, methods, and lambdas have an attached scope.
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

pub struct LambdaSignature {
    pub type_parameters: Vec<TypeParameter>,
    pub value_parameters: Vec<ValueParameter>,
    pub explicit_return_type_annotation: Option<TypeSymbolLookup>,
    pub ignorable: bool,
}

pub struct Lambda {
    pub signature: LambdaSignature,
    pub scope: Scope,
}

/// Parameterised modules are still being considered; until they're committed to, just a vector of
/// identifiers is enough. This is a perk of static methods not existing; there's no need to support
/// class-with-type-parameters components in the symbol lookup for now.
///
/// A lookup is an expression, but its information should be completely resolvable in the parsing
/// and semantic analysis. It allows looking items up in static program structure, e.g. types and
/// packages.
pub struct SymbolLookup(pub Vec<Identifier>);

pub enum Literal {
    Char(char),
    InterpolatedString(InterpolatedString),
    Number(i64, u64),
    String(SylanString),
    Lambda(Lambda),
}

pub struct Switch {
    pub expression: Box<Expression>,
    pub cases: Vec<Case>,
}

pub struct Timeout {
    pub nanoseconds: Box<Expression>,
    pub body: Scope,
}

pub struct Select {
    pub message_type: TypeSymbolLookup,
    pub cases: Vec<Case>,
    pub timeout: Option<Timeout>,
}

pub enum LambdaArgument {
    Normal(Argument<Expression>),
    Entry(Box<Expression>, Box<Expression>),
}

pub struct Call {
    pub target: Box<Expression>,
    pub arguments: Vec<LambdaArgument>,
}

pub struct Using(Box<Expression>);

pub struct If {
    pub condition: Box<Expression>,
    pub then: Scope,
    pub else_clause: Option<Scope>,
}

pub struct IfLet {
    pub binding: LocalBinding,
    pub then: Scope,
    pub else_clause: Option<Scope>,
}

pub struct CondCase {
    pub conditions: LinkedList<Expression>,
    pub then: Scope,
}

pub struct Cond(pub Vec<CondCase>);

pub struct CaseMatch {
    pub pattern: Pattern,
    pub guard: Option<Expression>,
}

pub struct Case {
    pub matches: LinkedList<CaseMatch>,
    pub body: Scope,
}

pub struct For {
    pub bindings: Vec<LocalBinding>,
    pub scope: Scope,
    pub label: Option<Identifier>,
}

pub struct While {
    pub condition: Box<Expression>,
    pub scope: Scope,
    pub label: Option<Identifier>,
}

pub struct WhileLet {
    pub binding: LocalBinding,
    pub scope: Scope,
    pub label: Option<Identifier>,
}

/// Throwing an expression does not yield a value as it destroys its current
/// process. However, it is an expression and can therefore be used anywhere an
/// expression can be used. It can throw any expression that yields a type which
/// implements the Exception interface. In "returns" the bottom type which
/// allows it to be used anywhere.
pub struct Throw(pub Box<Expression>);

pub struct PatternGetter {
    pub identifier: Identifier,
    pub pattern: Pattern,
}

pub struct CompositePattern {
    pub composite_type: TypeSymbolLookup,
    pub getters: Vec<PatternGetter>,
    pub ignore_rest: bool,
}

pub enum PatternItem {
    Literal(Literal),
    Identifier(Identifier),
    Composite(CompositePattern),
    Ignored,
}

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
