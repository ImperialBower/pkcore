#![allow(dead_code)]
// Dynamic Array Examples in Rust Structs

use std::collections::HashMap;

// 1. Vec<T> - Most common dynamic array approach
#[derive(Debug)]
struct DynamicVecStruct {
    name: String,
    data: Vec<i32>,
    capacity: usize,
}

impl DynamicVecStruct {
    fn new(name: String, initial_capacity: usize) -> Self {
        Self {
            name,
            data: Vec::with_capacity(initial_capacity),
            capacity: initial_capacity,
        }
    }

    fn add_item(&mut self, item: i32) {
        self.data.push(item);
    }

    fn get_size(&self) -> usize {
        self.data.len()
    }
}

// 2. Box<[T]> - Dynamically sized but fixed after creation
#[derive(Debug)]
struct BoxedArrayStruct {
    name: String,
    data: Box<[i32]>,
}

impl BoxedArrayStruct {
    fn new(name: String, size: usize, default_value: i32) -> Self {
        Self {
            name,
            data: vec![default_value; size].into_boxed_slice(),
        }
    }

    fn from_vec(name: String, vec: Vec<i32>) -> Self {
        Self {
            name,
            data: vec.into_boxed_slice(),
        }
    }

    fn get(&self, index: usize) -> Option<&i32> {
        self.data.get(index)
    }

    fn set(&mut self, index: usize, value: i32) -> Result<(), &'static str> {
        if index < self.data.len() {
            self.data[index] = value;
            Ok(())
        } else {
            Err("Index out of bounds")
        }
    }

    fn len(&self) -> usize {
        self.data.len()
    }
}

// 3. Generic const approach (if you know sizes at compile time)
#[derive(Debug)]
struct GenericArrayStruct<const N: usize> {
    name: String,
    data: [i32; N],
    current_index: usize,
}

impl<const N: usize> GenericArrayStruct<N> {
    fn new(name: String) -> Self {
        Self {
            name,
            data: [0; N],
            current_index: 0,
        }
    }

    fn add_item(&mut self, item: i32) -> Result<(), &'static str> {
        if self.current_index < N {
            self.data[self.current_index] = item;
            self.current_index += 1;
            Ok(())
        } else {
            Err("Array is full")
        }
    }

    fn get(&self, index: usize) -> Option<&i32> {
        if index < self.current_index {
            Some(&self.data[index])
        } else {
            None
        }
    }

    fn len(&self) -> usize {
        self.current_index
    }

    fn capacity(&self) -> usize {
        N
    }
}

// 4. Enum-based approach for different sized arrays
#[derive(Debug)]
enum DynamicSizedArray {
    Small([i32; 4]),
    Medium([i32; 16]),
    Large([i32; 64]),
    VeryLarge(Vec<i32>),
}

impl DynamicSizedArray {
    fn new_for_size(expected_size: usize) -> Self {
        match expected_size {
            0..=4 => Self::Small([0; 4]),
            5..=16 => Self::Medium([0; 16]),
            17..=64 => Self::Large([0; 64]),
            _ => Self::VeryLarge(Vec::with_capacity(expected_size)),
        }
    }

    fn set(&mut self, index: usize, value: i32) -> Result<(), &'static str> {
        match self {
            Self::Small(arr) => {
                if index < arr.len() {
                    arr[index] = value;
                    Ok(())
                } else {
                    Err("Index out of bounds")
                }
            }
            Self::Medium(arr) => {
                if index < arr.len() {
                    arr[index] = value;
                    Ok(())
                } else {
                    Err("Index out of bounds")
                }
            }
            Self::Large(arr) => {
                if index < arr.len() {
                    arr[index] = value;
                    Ok(())
                } else {
                    Err("Index out of bounds")
                }
            }
            Self::VeryLarge(vec) => {
                if index < vec.len() {
                    vec[index] = value;
                    Ok(())
                } else if index == vec.len() {
                    vec.push(value);
                    Ok(())
                } else {
                    Err("Index too far ahead")
                }
            }
        }
    }

    fn get(&self, index: usize) -> Option<&i32> {
        match self {
            Self::Small(arr) => arr.get(index),
            Self::Medium(arr) => arr.get(index),
            Self::Large(arr) => arr.get(index),
            Self::VeryLarge(vec) => vec.get(index),
        }
    }

    fn len(&self) -> usize {
        match self {
            Self::Small(arr) => arr.len(),
            Self::Medium(arr) => arr.len(),
            Self::Large(arr) => arr.len(),
            Self::VeryLarge(vec) => vec.len(),
        }
    }
}

#[derive(Debug)]
struct AdaptiveStruct {
    name: String,
    data: DynamicSizedArray,
}

impl AdaptiveStruct {
    fn new(name: String, expected_size: usize) -> Self {
        Self {
            name,
            data: DynamicSizedArray::new_for_size(expected_size),
        }
    }
}

// 5. Builder pattern for dynamic construction
#[derive(Debug)]
struct BuilderArrayStruct {
    name: String,
    data: Vec<i32>,
    metadata: HashMap<String, String>,
}

struct BuilderArrayStructBuilder {
    name: Option<String>,
    capacity: Option<usize>,
    initial_data: Vec<i32>,
    metadata: HashMap<String, String>,
}

impl BuilderArrayStructBuilder {
    fn new() -> Self {
        Self {
            name: None,
            capacity: None,
            initial_data: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    fn name(mut self, name: String) -> Self {
        self.name = Some(name);
        self
    }

    fn capacity(mut self, capacity: usize) -> Self {
        self.capacity = Some(capacity);
        self
    }

    fn add_data(mut self, data: Vec<i32>) -> Self {
        self.initial_data.extend(data);
        self
    }

    fn add_item(mut self, item: i32) -> Self {
        self.initial_data.push(item);
        self
    }

    fn add_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }

    fn build(self) -> Result<BuilderArrayStruct, &'static str> {
        let name = self.name.ok_or("Name is required")?;

        let mut data = if let Some(cap) = self.capacity {
            Vec::with_capacity(cap)
        } else {
            Vec::new()
        };

        data.extend(self.initial_data);

        Ok(BuilderArrayStruct {
            name,
            data,
            metadata: self.metadata,
        })
    }
}

impl BuilderArrayStruct {
    fn builder() -> BuilderArrayStructBuilder {
        BuilderArrayStructBuilder::new()
    }
}

fn main() {
    println!("=== 1. Vec<T> - Most flexible dynamic arrays ===");
    let mut vec_struct = DynamicVecStruct::new("Dynamic Vec".to_string(), 10);
    vec_struct.add_item(1);
    vec_struct.add_item(2);
    vec_struct.add_item(3);
    println!("{:?}", vec_struct);
    println!("Size: {}", vec_struct.get_size());

    println!("\n=== 2. Box<[T]> - Fixed size after creation ===");
    let mut boxed_struct = BoxedArrayStruct::new("Boxed Array".to_string(), 5, 42);
    boxed_struct.set(0, 100).unwrap();
    boxed_struct.set(4, 500).unwrap();
    println!("{:?}", boxed_struct);
    println!("Length: {}", boxed_struct.len());

    let from_vec = BoxedArrayStruct::from_vec("From Vec".to_string(), vec![1, 2, 3, 4, 5, 6, 7]);
    println!("From vec: {:?}", from_vec);

    println!("\n=== 3. Generic const arrays ===");
    let mut small_array: GenericArrayStruct<4> = GenericArrayStruct::new("Small".to_string());
    small_array.add_item(10).unwrap();
    small_array.add_item(20).unwrap();
    println!("{:?}", small_array);
    println!("Capacity: {}, Used: {}", small_array.capacity(), small_array.len());

    let mut large_array: GenericArrayStruct<100> = GenericArrayStruct::new("Large".to_string());
    for i in 0..10 {
        large_array.add_item(i * 10).unwrap();
    }
    println!("Large array used: {}/{}", large_array.len(), large_array.capacity());

    println!("\n=== 4. Enum-based size adaptation ===");
    let mut adaptive_small = AdaptiveStruct::new("Small adaptive".to_string(), 3);
    adaptive_small.data.set(0, 100).unwrap();
    println!("{:?}", adaptive_small);

    let mut adaptive_large = AdaptiveStruct::new("Large adaptive".to_string(), 100);
    adaptive_large.data.set(0, 1000).unwrap();
    println!(
        "Large adaptive data type: {:?}",
        std::mem::discriminant(&adaptive_large.data)
    );

    println!("\n=== 5. Builder pattern ===");
    let builder_struct = BuilderArrayStruct::builder()
        .name("Built Structure".to_string())
        .capacity(20)
        .add_item(1)
        .add_item(2)
        .add_data(vec![3, 4, 5])
        .add_metadata("type".to_string(), "example".to_string())
        .build()
        .unwrap();

    println!("{:?}", builder_struct);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vec_struct() {
        let mut s = DynamicVecStruct::new("test".to_string(), 5);
        assert_eq!(s.get_size(), 0);
        s.add_item(42);
        assert_eq!(s.get_size(), 1);
    }

    #[test]
    fn test_boxed_array() {
        let mut s = BoxedArrayStruct::new("test".to_string(), 3, 0);
        assert_eq!(s.len(), 3);
        s.set(1, 42).unwrap();
        assert_eq!(*s.get(1).unwrap(), 42);
    }

    #[test]
    fn test_generic_array() {
        let mut s: GenericArrayStruct<5> = GenericArrayStruct::new("test".to_string());
        assert_eq!(s.capacity(), 5);
        assert_eq!(s.len(), 0);

        s.add_item(1).unwrap();
        s.add_item(2).unwrap();
        assert_eq!(s.len(), 2);
        assert_eq!(*s.get(0).unwrap(), 1);
    }

    #[test]
    fn test_adaptive_array() {
        let small = DynamicSizedArray::new_for_size(2);
        let large = DynamicSizedArray::new_for_size(100);

        // Different variants should be created
        assert_ne!(std::mem::discriminant(&small), std::mem::discriminant(&large));
    }

    #[test]
    fn test_builder() {
        let result = BuilderArrayStruct::builder()
            .name("test".to_string())
            .add_item(42)
            .build();

        assert!(result.is_ok());
        let s = result.unwrap();
        assert_eq!(s.name, "test");
        assert_eq!(s.data[0], 42);
    }
}
