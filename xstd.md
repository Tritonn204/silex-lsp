
# Complete Breakdown of Data

## Key: `Some(Array(T(0)))`
Functions for arrays of generic type `T(0)`:

1. **`len`**:
   - Parameters: None
   - Return Type: `U32`

2. **`slice`**:
   - Parameters: `range: Range(U32)`
   - Return Type: `Array(T(0))`

3. **`push`**:
   - Parameters: `value: T(0)`
   - Return Type: None

4. **`get`**:
   - Parameters: `index: U32`
   - Return Type: `Optional(T(0))`

5. **`contains`**:
   - Parameters: `value: T(0)`
   - Return Type: `Bool`

6. **`pop`**:
   - Parameters: None
   - Return Type: `Optional(T(0))`

7. **`first`**:
   - Parameters: None
   - Return Type: `Optional(T(0))`

8. **`last`**:
   - Parameters: None
   - Return Type: `Optional(T(0))`

9. **`remove`**:
   - Parameters: `index: U32`
   - Return Type: `T(0)`

---

## Key: `Some(U64)`
Functions for `U64` (64-bit unsigned integer):

1. **`to_le_bytes`**:
   - Parameters: None
   - Return Type: `Array(U8)`

2. **`to_be_bytes`**:
   - Parameters: None
   - Return Type: `Array(U8)`

3. **`overflowing_add`**:
   - Parameters: `other: U64`
   - Return Type: `Optional(U64)`

4. **`overflowing_sub`**:
   - Parameters: `other: U64`
   - Return Type: `Optional(U64)`

5. **`overflowing_mul`**:
   - Parameters: `other: U64`
   - Return Type: `Optional(U64)`

6. **`overflowing_div`**:
   - Parameters: `other: U64`
   - Return Type: `Optional(U64)`

7. **`overflowing_rem`**:
   - Parameters: `other: U64`
   - Return Type: `Optional(U64)`

---

## Key: `Some(U128)`
Functions for `U128` (128-bit unsigned integer):

- Same functions as `U64` (`to_le_bytes`, `to_be_bytes`, and `overflowing_*`) with `U128` types.

---

## Key: `Some(U256)`
Functions for `U256` (256-bit unsigned integer):

- Same functions as `U64` and `U128`, but with `U256` types.

---

## Key: `Some(U16)`
Functions for `U16` (16-bit unsigned integer):

- Same functions as `U64`, but with `U16` types.

---

## Key: `Some(U32)`
Functions for `U32` (32-bit unsigned integer):

- Same functions as `U64`, but with `U32` types.

---

## Key: `Some(Range(T(0)))`
Functions for a range of type `T(0)`:

1. **`max`**:
   - Parameters: None
   - Return Type: `T(0)`

2. **`min`**:
   - Parameters: None
   - Return Type: `T(0)`

3. **`contains`**:
   - Parameters: `value: T(0)`
   - Return Type: `Bool`

4. **`collect`**:
   - Parameters: None
   - Return Type: `Array(T(0))`

5. **`count`**:
   - Parameters: None
   - Return Type: `T(0)`

---

## Key: `Some(U8)`
Functions for `U8` (8-bit unsigned integer):

- Same functions as `U64`, but with `U8` types.

---

## Key: `None`
Global functions:

1. **`require`**:
   - Parameters: `value: Bool`
   - Return Type: None

2. **`assert`**:
   - Parameters: `value: Bool`
   - Return Type: None

3. **`panic`**:
   - Parameters: `value: Any`
   - Return Type: `Any`

4. **`println`**:
   - Parameters: `value: Any`
   - Return Type: None

5. **`is_same_ptr`**:
   - Parameters:
     - `left: Any`
     - `right: Any`
   - Return Type: `Bool`

6. **`debug`**:
   - Parameters: `value: Any`
   - Return Type: None

---

## Key: `Some(Optional(T(0)))`
Functions for optional values of type `T(0)`:

1. **`expect`**:
   - Parameters: `msg: String`
   - Return Type: `T(0)`

2. **`unwrap_or`**:
   - Parameters: `default: T(0)`
   - Return Type: `T(0)`

3. **`is_none`**:
   - Parameters: None
   - Return Type: `Bool`

4. **`unwrap`**:
   - Parameters: None
   - Return Type: `T(0)`

5. **`is_some`**:
   - Parameters: None
   - Return Type: `Bool`

---

## Key: `Some(String)`
Functions for strings:

1. **`len`**:
   - Parameters: None
   - Return Type: `U32`

2. **`contains`**:
   - Parameters: `value: String`
   - Return Type: `Bool`

3. **`split`**:
   - Parameters: `at: String`
   - Return Type: `Array(String)`

4. **`to_bytes`**:
   - Parameters: None
   - Return Type: `Array(U8)`

5. **`index_of`**:
   - Parameters: `value: String`
   - Return Type: `Optional(U32)`

6. **`replace`**:
   - Parameters:
     - `from: String`
     - `to: String`
   - Return Type: `String`

7. **`substring`**:
   - Parameters:
     - `start: U32`
     - `end: U32`
   - Return Type: `Optional(String)`

8. **`is_empty`**:
   - Parameters: None
   - Return Type: `Bool`

9. **`char_at`**:
   - Parameters: `index: U32`
   - Return Type: `Optional(String)`

10. **`to_uppercase`**:
    - Parameters: None
    - Return Type: `String`

11. **`to_lowercase`**:
    - Parameters: None
    - Return Type: `String`

---

## Key: `Some(Map(T(0), T(1)))`
Functions for maps with keys of type `T(0)` and values of type `T(1)`:

1. **`contains_key`**:
   - Parameters: `key: T(0)`
   - Return Type: `Bool`

2. **`len`**:
   - Parameters: None
   - Return Type: `U32`

3. **`keys`**:
   - Parameters: None
   - Return Type: `Array(T(0))`

4. **`insert`**:
   - Parameters:
     - `key: T(0)`
     - `value: T(1)`
   - Return Type: `Optional(T(1))`

5. **`get`**:
   - Parameters: `key: T(0)`
   - Return Type: `Optional(T(1))`

6. **`values`**:
   - Parameters: None
   - Return Type: `Array(T(1))`
