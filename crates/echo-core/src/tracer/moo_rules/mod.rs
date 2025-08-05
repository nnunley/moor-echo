/*!
# MOO-Specific Transformation Rules

Collection of transformation rules specific to MOO-to-Echo migration
and MOO compatibility improvements.
*/

pub mod property_syntax;
pub mod object_refs;
pub mod builtin_functions;

// Re-export commonly used rules
pub use property_syntax::PropertySyntaxFixer;
pub use object_refs::ObjectReferenceNormalizer;
pub use builtin_functions::BuiltinFunctionResolver;