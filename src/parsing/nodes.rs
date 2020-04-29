//! Sylan consists of items and expressions. Items are declarative whereas
//! expressions are executed and yield values. Such values can be the void value
//! for expressions executed solely for side-effects. "Statements" can be
//! approximated by stacking expressions one after the other and discarding
//! their values.
//!
//! Items are guaranteed to be evaluatable in constant-time. Importing a
//! package should never take an indefinite amount of time due to side effects
//! being invoked. That guarantee is not upheld for _compiling_ them, however.
//!
//! Patterns are used throughout Sylan. They are either refuttable or
//! irrefuttable. Here is where each can be used:
//!
//! * Refuttable: `switch` cases, `select` cases, `while var`, and
//!   `if var`. `switch` and `select` also add guard clauses, which make
//!   any pattern refutable unless it yields compile-time boolean true value.
//! * Irrefuttable: anywhere else, namely: var bindings, final bindings, `for`
//!   bindings, and parameters.
//!
//! Here's how patterns are grouped into refuttable or irrefuttable, based
//! solely on the pattern itself and not what is being assigned into it. The
//! exception to that is compile-time constants which consider both sides for
//! refuttability, but will subsequently reject such a pattern for being a
//! pointless no-op.
//!
//! * Irrefuttable: identifiers, including the ignored `_` identifier,
//!   Types with irrefutable patterns in all fields, recursively, _except_ for
//!   enum variants (which are technically type constructors and not types),
//!   and compile-time constants where the assigned value is also a matching
//!   compile-time constant (which is rejected by the compiler later on for
//!   being redundent, but is still technically a irefuttable pattern).
//! * Refuttable: switch and select clauses with guard patterns (which are
//!   technically part of switch and select and not part of patterns), enum
//!   variants, values resolved from identifier with `.`, and literals in the
//!   pattern that are either not compile-time (e.g. interpolated strings), or
//!   cannot be matched with an equivalently compile-time equivalent in the
//!   right hand side.
//!
//! Contexts that expect refuttable patterns will reject irrefutable patterns,
//! and vice-versa. Reffutable patterns used as irrefuttable paterns are
//! unsound, and irreffutable patterns as refuttable patterns are pointless.
//!
//! The parser doesn't care, since refuttabillity can only be asserted with a
//! type system. Thus, they are both just "patterns" here.

use std::rc::Rc;

use crate::common::multiphase::{
    Accessibility, Identifier, InterpolatedString, OverloadableInfixOperator, PostfixOperator,
    PseudoIdentifier, Shebang, SyDoc, SylanString,
};
use crate::common::version::Version;

/// Shebangs and source versions are special, which is why they're outside of
/// the `PackageFile` in which all other items and expressions reside. Both
/// shebangs must be completely resolved before anything else can be parsed,
/// and the result of parsing version can completely change the lexing and
/// parsing of all subsequent tokens.
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct File {
    pub shebang: Option<Shebang>,
    pub version: Option<Version>,
    pub package: Package,
}

/// Main files are the files that are directly invoked by Sylan. They have
/// abilities that imported files to not; see [MainPackage] for more details.
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct MainFile {
    pub shebang: Option<Shebang>,
    pub version: Option<Version>,
    pub package: MainPackage,
}

// Packages only have items at top-level, with the exception of the main package that can also have
// executable code to simplify small scripts.

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Package {
    pub imports: Vec<Import>,
    pub accessibility: Accessibility,
    pub name: Identifier,
    pub items: Vec<Item>,
    pub sydoc: Option<SyDoc>,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct MainPackage {
    pub package: Package,
    pub block: Block,
}

/// Every node in Sylan is either an item or an expression, even the special
/// shebang and version tokens (both of which are items).
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum Node {
    Item(Item),
    Expression(Expression),
}

/// The declarations that make up the static structure of a Sylan program. Items
/// can't be contained within expressions, with the exception of bindings.
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum Item {
    Extension(Extension),
    Fun(Fun),
    Package(Package),
    Type(Type),
    Final(Final),

    // Unlike the previous variants, these can be arbitrarily nested within
    // expressions. This is to allow corecursion among other features.
    //
    // For loops also create bindings, but are not items because I can't
    // think of a use case for mutually recursive loop continuation bindings.
    Binding(Binding),
}

/// The expressions that allow Turing-complete computations, i.e. allowing
/// Sylan to do actual useful work.
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum Expression {
    BranchingAndJumping(BranchingAndJumping),
    Context(Block),
    Literal(Literal),
    Operator(Operator),
    Symbol(Symbol),
    Throw(Throw),
    Using(Using),
    NonDestructiveUpdate(Call),
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum Operator {
    InfixOperator(OverloadableInfixOperator, Box<Expression>, Box<Expression>),
    PostfixOperator(Box<Expression>, PostfixOperator),
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum BranchingAndJumping {
    Call(Call),
    Cond(Cond),
    For(For),
    If(If),
    IfVar(IfVar),
    Select(Select),
    Switch(Switch),
    While(While),
    WhileVar(WhileVar),
}

// One notable difference between funs and lambdas is that omitting a return
// type on a lambda triggers type inference, whereas it always means the `Void`
// type for `fun`. Also, `fun` expects its signature to explicitly type every
// parameter. This is because `fun` is intended to be used for top-level
// functions that define the shape of the program or API, in which types should
// be explicitly annotated anyway. It also adds a bounds to potentially
// expensive type inference costs in the compiler.

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct FunModifiers {
    pub accessibility: Accessibility,
    pub is_extern: bool,
    pub is_operator: bool,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct ValueParameter {
    /// A label is omitted if the developer passes an `_` where a label is expected.
    /// By constrast, if a label is totally omitted, it assumes the same
    /// identifier as the parameter. If the parameter uses pattern-matching
    /// beyond a basic identifier, this stops working, and the compiler notifies
    /// them that they must provide an explicit label or use a basic identifier as
    /// the parameter pattern.
    ///
    /// The same applies to lambdas and enum variants.
    pub label: Option<Identifier>,

    /// TODO: tolerate any token or grouped token to tolerate procedural macros.
    pub is_syntax: bool,

    pub pattern: Pattern,
    pub type_annotation: TypeReference,
    pub default_value: Option<Expression>,
    pub sydoc: Option<SyDoc>,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct ClassValueParameterFieldUpgrade {
    pub is_embedded: bool,
    pub accessibility: Accessibility,
}

/// The same except as a [ValueParameter] except that they can be upgraded to
/// fields by prefixing with var with the usual field modifiers.
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct ClassValueParameter {
    pub parameter: ValueParameter,
    pub field_upgrade: Option<ClassValueParameterFieldUpgrade>,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct ReturnType {
    r#type: TypeReference,
    ignorable: bool,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct FunSignature {
    pub name: Identifier,
    pub sydoc: Option<SyDoc>,
    pub type_parameters: Vec<TypeParameter>,
    pub value_parameters: Vec<ValueParameter>,

    // Unlike lambdas, an empty return type does not fallback to inference.
    // Instead, `Void` is assumed.
    pub return_type: Option<ReturnType>,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Fun {
    pub modifiers: FunModifiers,
    pub signature: FunSignature,
    pub block: Block,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct ImportSingleStem {
    pub name: Identifier,

    // Will be empty for the vast majority of imports.
    pub readers: Vec<Symbol>,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum ImportStem {
    Single(ImportSingleStem),
    Multiple(Vec<Import>),
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Import {
    pub root: Option<Symbol>,
    pub stem: ImportStem,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum DeclarationItem {
    Binding(Binding),
    Type(Type),
    Package(Package),
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Declaration {
    pub accessibility: Accessibility,
    pub item: DeclarationItem,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct ClassModifiers {
    accessibility: Accessibility,
    is_extern: bool,
}

// Concrete classes that support implementing interfaces and embedding other
// classes, but cannot extend other classes directly.

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Class {
    pub implements: Vec<TypeReference>,
    pub methods: Vec<ConcreteMethod>,
    pub fields: Vec<Field>,

    // Initialisation
    pub value_parameters: Vec<ClassValueParameter>,
    pub instance_initialiser: Block,
}

/// Enum variants look and feel like function parameter lists, but default
/// values and arbitrarily deep pattern matching are omitted because they
/// don't make sense specifically for enum variants. Defaults are dropped,
/// and the pattern matching is restricted to just an identifier, i.e. a
/// simple parameter name.
///
/// Labels can still be used, however.
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct EnumVariant {
    pub label: Option<Identifier>,
    pub name: Identifier,
    pub type_annotation: TypeReference,
    pub sydoc: Option<SyDoc>,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Enum {
    pub variants: Vec<EnumVariant>,
    pub class: Class,
}

/// Interfaces that support extending other interfaces, providing empty methods
/// that implementors must implement, providing already-defined utility methods,
/// and even allowing already-defined methods to be specialised via overriding
/// in implementing classes.
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Interface {
    pub extends: Vec<TypeReference>,
    pub methods: Vec<Method>,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum TypeItem {
    Class(Class),
    Enum(Enum),
    Interface(Interface),
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Type {
    pub name: Identifier,
    pub type_parameters: Vec<TypeParameter>,
    pub item: TypeItem,
    pub sydoc: Option<SyDoc>,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct TypeReference {
    pub symbol: Symbol,
    pub type_arguments: Vec<TypeArgument>,
}

impl TypeReference {
    pub fn new(symbol: Symbol) -> Self {
        Self {
            symbol,
            type_arguments: vec![],
        }
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Extension {
    pub symbol: Symbol,
    pub extension_parameters: Vec<TypeParameter>,
    pub type_parameters: Vec<TypeParameter>,
    pub item: Class,
    pub sydoc: Option<SyDoc>,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct MethodModifiers {
    fun_modifiers: FunModifiers,
    overrides: bool,
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

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct AbstractMethod {
    pub modifiers: MethodModifiers,
    pub signature: FunSignature,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct ConcreteMethod {
    r#abstract: AbstractMethod,
    scope: Block,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum Method {
    Abstract(AbstractMethod),
    Concrete(ConcreteMethod),
}

/// Type parameters are for types at compile-time and have optional upper
/// bounds, identifiers, and optional default values.
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct TypeParameter {
    pub label: Option<Identifier>,
    pub name: Identifier,
    pub upper_bounds: Vec<TypeReference>,
    pub default_value: Option<TypeReference>,
    pub sydoc: Option<SyDoc>,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Argument<T> {
    pub label: Option<Identifier>,
    pub value: T,
}

/// Value arguments are for values at runtime. They support being passed as
/// positional or keyword arguments; unlike other languages it is the choice of
/// the caller rather than the definer. If passed as a keyword argument, an
/// identifier is carried with it in the parse tree.
pub type ValueArgument = Argument<Expression>;

/// Type arguments are for values at runtime. They support being passed as
/// positional or keyword arguments; unlike other languages it is the choice of
/// the caller rather than the definer. If passed as a keyword argument, an
/// identifier is carried with it in the parse tree.
pub type TypeArgument = Argument<Type>;

// Sylan's "symbol tables" are just a collection of bindings in the current
// scope. Parent scopes can be looked up to find bindings in outer closures,
// which is how lexical scoping is implemented.

/// There are three types of variable: local bindings, fields, and constants.
///
/// Local bindings don't have modifiers or SyDocs, whereas fields and constants
/// can. Constants appear in the top-level of non-main packages, but fields and
/// local bindings cannot.
///
/// Constants can only have compile-time values; they guarantee no runtime hit.
/// By extension, this means that all package imports are guaranteed to be
/// constant-time and have no side-effects.
///
/// Bindings are for execution-time values. Statically deducible values go via
/// package, types definitions, and constants instead. (Note that
/// "execution-time" can mean both "runtime" and "running within a compile-time
/// macro.)

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Binding {
    pub pattern: Pattern,
    pub value: Box<Expression>,
    pub explicit_type_annotation: Option<TypeReference>,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Final {
    /// This is gaping escape hatch from Sylan's immutable worldview. If a extern
    /// final value points to a memory location in another artefact, like a C
    /// dynamic library, it can actually mutate the variable underneath Sylan.
    ///
    /// Sylan therefore treats all extern finals as volatile, forcing memory
    /// fences on loading and storing, unless the `nonvolatile` keyword is used.
    /// The usual tricks such as automatic caching of the value in higher levels
    /// are also dropped.
    ///
    /// It might be helpful to think of `final` as "final from Sylan's
    /// perspective", not "won't ever change".
    ///
    /// Modifying such values, like modifying any memory location directly in
    /// Sylan, will require unsafe FFI APIs.
    ///
    /// This pollution of Sylan's immutable world view cascades downwards; any
    /// function relying on its value can no longer assume the same output even
    /// without `select` and with consistent inputs, so such optimisations also
    /// are discarded. This pollution also applies to function calling extern
    /// functions, using select, or using other functions that have these same
    /// traits.
    pub is_extern: bool,

    pub accessibility: Accessibility,
    pub binding: Binding,
    pub sydoc: Option<SyDoc>,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Field {
    pub is_extern: bool,
    pub is_embedded: bool,
    pub accessibility: Accessibility,
    pub binding: Binding,
}

/// Expressions are seperate from bindings.
type Expressions = Vec<Expression>;

/// Bindings within a block are resolved before executing its
/// expressions, which is why they're items rather than expressions.
/// This is to allow techniques like mutual recursion and
/// self-referential functions and methods without forward declarations.
///
/// Blocks are themselves scopes; the two can't be meaningfully separated in
/// Sylan.
///
/// Note that the declarations aren't accessible until their declarations have
/// been executed, but don't cause compilation problems if accessed within a
/// delayed computation within the same scope.
///
/// In other words, these declarations are block-scoped with a temporal dead
/// zone rather than using scope hoisting, to use JavaScript terminology.
///
/// Blocks, unlike non-main packages, can contain executable code. Being an
/// expression-oriented language, executable code is just a sequence of
/// expressions. Unlike non-main packages, they can refer to parent blocks to
/// provide scope lookups. They can declare variables with bindings but cannot
/// declare new types like packages can.
///
/// All functions, concrete methods, and lambdas have an attached scope.

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Block {
    pub bindings: Vec<Binding>,
    pub expressions: Expressions,
    pub parent: Option<Rc<Block>>,
}

impl Block {
    pub fn new_root() -> Self {
        Block {
            bindings: vec![],
            expressions: vec![],
            parent: None,
        }
    }

    pub fn within(parent: &Rc<Block>) -> Self {
        Block {
            bindings: vec![],
            expressions: vec![],
            parent: Some(parent.clone()),
        }
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct LambdaValueParameter {
    pub label: Option<Identifier>,
    pub pattern: Pattern,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct LambdaSignature {
    pub value_parameters: Vec<LambdaValueParameter>,
    // Non-void lambda results can always be ignored without warnings, hence no
    // `ignorable` modifier. Sylan is only concerned if declared top-level
    // functions in an API are ignored without declaring such an ignoral to be
    // acceptable.
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Lambda {
    pub signature: LambdaSignature,
    pub block: Block,
}

// Parameterised modules are still being considered; until they're committed to, just a vector of
// identifiers is enough. Static methods don't exist in Sylan, but `Class.method` as syntactical
// sugar for `-> object, ..args { object.method(..args)}` does, so type symbols must also be
// allowed (albeit without type parameters, which are solely inferred in this context).
//
// A lookup is an expression, but its information should be completely resolvable in the parsing
// and semantic analysis. It allows looking items up in static program structure, e.g. types and
// packages.

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct SymbolLookup(pub Vec<Identifier>);

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum Symbol {
    Relative(SymbolLookup),
    Absolute(SymbolLookup),
    Pseudo(PseudoIdentifier),
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum Literal {
    Char(char),
    InterpolatedString(InterpolatedString),
    Number(i64, u64),
    String(SylanString),
    Lambda(Lambda),
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Switch {
    pub expression: Box<Expression>,
    pub cases: Vec<Case>,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Timeout {
    pub nanoseconds: Box<Expression>,
    pub body: Block,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Select {
    pub message_type: TypeReference,
    pub cases: Vec<Case>,
    pub timeout: Option<Timeout>,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Call {
    pub target: Box<Expression>,
    pub type_arguments: Vec<TypeArgument>,
    pub arguments: Vec<ValueArgument>,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Using(Box<Expression>);

// Ifs must have braces for both the matching body and the else clause if one
// exists, like any other control statement. There's one exception: if the else
// is followed directly by another `if`, the braces can be dropped. This is to
// allow the common `} else if {` notation.
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct If {
    pub condition: Box<Expression>,
    pub then: Block,
    pub else_clause: Option<Block>,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct IfVar {
    pub binding: Binding,
    pub then: Block,
    pub else_clause: Option<Block>,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct CondCase {
    pub conditions: Vec<Expression>,
    pub then: Block,
}

// The first expression yielding true is used; all others are ignored and not
// even evaluated, regardless of whether they'd also return true.
//
// Any expression not yielding a `Boolean` type fails type checking.
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Cond(pub Vec<CondCase>);

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct CaseMatch {
    pub pattern: Pattern,
    pub guard: Option<Expression>,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Case {
    pub matches: Vec<CaseMatch>,
    pub body: Block,
}

// For loop "labels" are completely different to parameter labels. They are
// instead names of continuation identifiers that get predefined inside for loop
// bodies. They are inspired semantically by both Scheme's named-let and
// syntactically by Java's loop break labels.
//
// * https://docs.racket-lang.org/guide/let.html#%28part._.Named_let%29
// * https://docs.oracle.com/javase/tutorial/java/nutsandbolts/branch.html
//
// For loops will halt unless continue or a label is called.
//
// `if var`, `while var`, and `for while` all allow multiple `var` bindings,
// separated with commas. `if var` and a `while var` expect _all_ bindings to
// match before continuing into the block. `for` won't allow refutable
// patterns; refuttable patterns must be done inside the for loop with other
// constructs.

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct For {
    pub bindings: Vec<Binding>,
    pub scope: Block,
    pub label: Option<Identifier>,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct While {
    pub condition: Box<Expression>,
    pub scope: Block,
}

// `while var` does not accept labels. If a developers need that, they should
// use for loops instead, and perform refuttable pattern matching against the
// irefuttable pattern bound by `for` inside the body.
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct WhileVar {
    pub binding: Binding,
    pub scope: Block,
}

/// Throwing an expression does not yield a value as it destroys its current
/// process. However, it is an expression and can therefore be used anywhere an
/// expression can be used. It can throw any expression that yields a type which
/// implements the Exception interface. In "returns" the bottom type which
/// allows it to be used anywhere.
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Throw(pub Box<Expression>);

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct PatternGetter {
    pub label: Option<Identifier>,
    pub name: Identifier,
    pub pattern: Pattern,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct CompositePattern {
    pub r#type: TypeReference,
    pub getters: Vec<PatternGetter>,
    pub infer_enum_type: bool,
    pub ignore_rest: bool,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum PatternItem {
    // Irrefuttable
    Identifier(Identifier),
    Ignored,

    // Irrefutable unless an interpolated string is used.
    Literal(Literal),

    // Irrefutable if all fields are also refuttable.
    Composite(CompositePattern),

    // Refuttable, as it's worked out at runtime from what the symbol resolves
    // to. Irrefuttable if it can be resolved at compile-time _and_ the
    // left-hand side can also be resolved at compile-time.
    BoundSymbol(Symbol),
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Pattern {
    pub item: PatternItem,

    // Bound with `as`, a ireffutable pattern match on the right hand side and
    // available in following-on blocks such as switch/select clauses and
    // guards, fun bodies, and `if let`, `while let`, and `for` blocks.
    pub bound_match: Option<Box<Pattern>>,
}
