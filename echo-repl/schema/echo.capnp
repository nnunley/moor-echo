@0x85150b117366d14a;

# Echo object schema for Cap'n Proto serialization

struct ObjectId {
    uuid @0 :Data;  # 16 bytes UUID
}

struct PropertyValue {
    union {
        null @0 :Void;
        boolean @1 :Bool;
        integer @2 :Int64;
        float @3 :Float64;
        string @4 :Text;
        object @5 :ObjectId;
        list @6 :List(PropertyValue);
        map @7 :List(MapEntry);
    }
}

struct MapEntry {
    key @0 :Text;
    value @1 :PropertyValue;
}

struct VerbSignature {
    dobj @0 :Text;
    prep @1 :Text;
    iobj @2 :Text;
}

struct VerbPermissions {
    read @0 :Bool;
    write @1 :Bool;
    execute @2 :Bool;
}

struct VerbDefinition {
    name @0 :Text;
    signature @1 :VerbSignature;
    code @2 :Text;
    permissions @3 :VerbPermissions;
}

struct PropertyMetadata {
    name @0 :Text;
    owner @1 :ObjectId;
    flags @2 :UInt32;
    permissions @3 :UInt32;
}

struct VerbMetadata {
    name @0 :Text;
    owner @1 :ObjectId;
    flags @2 :UInt32;
    permissions @3 :UInt32;
}

struct EventMetadata {
    name @0 :Text;
    handler @1 :Text;
    priority @2 :Int32;
}

struct QueryMetadata {
    name @0 :Text;
    query @1 :Text;
    cached @2 :Bool;
}

struct GreenThreadId {
    uuid @0 :Data;  # 16 bytes UUID
}

struct MetaObject {
    id @0 :ObjectId;
    propertyMetadata @1 :List(PropertyMetadata);
    verbMetadata @2 :List(VerbMetadata);
    eventMetadata @3 :List(EventMetadata);
    queryMetadata @4 :List(QueryMetadata);
    greenThreads @5 :List(GreenThreadId);
    lastModified @6 :UInt64;  # Unix timestamp
    version @7 :UInt32;
}

struct EchoObject {
    id @0 :ObjectId;
    parent @1 :ObjectId;  # Use null UUID for None
    name @2 :Text;
    properties @3 :List(Property);
    verbs @4 :List(VerbDefinition);
    queries @5 :List(Query);
    meta @6 :MetaObject;
}

struct Property {
    name @0 :Text;
    value @1 :PropertyValue;
}

struct Query {
    name @0 :Text;
    code @1 :Text;
}