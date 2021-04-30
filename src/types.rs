use linked_hash_map::LinkedHashMap;

#[derive(Debug, Clone)]
pub enum Type {
    Boolean,
    Number,
    Void,
    Undefined,
    Array {
        element_type: Box<Type>,
    },
    Function {
        parameter_types: LinkedHashMap<String, Type>,
        return_type: Box<Type>,
    },
}

impl PartialEq for Type {
    fn eq(&self, rhs: &Type) -> bool {
        match (self, rhs) {
            (Type::Boolean, Type::Boolean) => true,
            (Type::Number, Type::Number) => true,
            (Type::Undefined, Type::Undefined) => true,
            (Type::Void, Type::Void) => true,
            (Type::Array { element_type: a }, Type::Array { element_type: b }) => a == b,
            (
                Type::Function {
                    parameter_types: pa,
                    return_type: ra,
                },
                Type::Function {
                    parameter_types: pb,
                    return_type: rb,
                },
            ) => ra == rb && pa.values().into_iter().eq(pb.values().into_iter()),
            _ => false,
        }
    }
}

impl Eq for Type {}
