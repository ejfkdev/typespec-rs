// Namespace declarations and nested types
// expect-clean
namespace PetStore {
  model Pet {
    name: string;
  }

  enum Status {
    available,
    sold,
  }
}

namespace StoreV2 {
  model PetV2 {
    name: string;
    age: int32;
  }
}
