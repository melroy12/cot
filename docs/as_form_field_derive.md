# AsFormField Derive Macro

This derive macro provides a convenient way to automatically implement the `AsFormField` trait for enums that already implement `SelectChoice`. This drastically reduces boilerplate code when creating form select fields.

## Before (Manual Implementation)

```rust
#[derive(Debug, Clone, PartialEq)]
enum Priority {
    Low,
    Medium,
    High,
}

impl SelectChoice for Priority {
    fn default_choices() -> Vec<Self> {
        vec![Self::Low, Self::Medium, Self::High]
    }

    fn from_str(s: &str) -> Result<Self, FormFieldValidationError> {
        match s {
            "low" => Ok(Self::Low),
            "medium" => Ok(Self::Medium),
            "high" => Ok(Self::High),
            _ => Err(FormFieldValidationError::invalid_value(s.to_owned())),
        }
    }

    fn id(&self) -> String {
        match self {
            Self::Low => "low".to_string(),
            Self::Medium => "medium".to_string(),
            Self::High => "high".to_string(),
        }
    }

    fn to_string(&self) -> String {
        match self {
            Self::Low => "Low Priority".to_string(),
            Self::Medium => "Medium Priority".to_string(),
            Self::High => "High Priority".to_string(),
        }
    }
}

impl AsFormField for Priority {
    type Type = SelectField<Self>;

    fn clean_value(field: &Self::Type) -> Result<Self, FormFieldValidationError> {
        if let Some(value) = field.value() {
            if value.is_empty() {
                return Err(FormFieldValidationError::Required);
            }
            Self::from_str(value)
        } else {
            Err(FormFieldValidationError::Required)
        }
    }

    fn to_field_value(&self) -> String {
        self.id()
    }
}
```

## After (Using Derive Macros)

```rust
use cot_macros::{AsFormField, SelectChoice as DeriveSelectChoice};

#[derive(Debug, Clone, PartialEq, DeriveSelectChoice, AsFormField)]
enum Priority {
    #[select_choice(id = "low", name = "Low Priority")]
    Low,
    #[select_choice(id = "medium", name = "Medium Priority")]
    Medium,
    #[select_choice(id = "high", name = "High Priority")]
    High,
}
```

## Simple Enum with Defaults

For enums where the variant name can be used as both the ID and display name:

```rust
#[derive(Debug, Clone, PartialEq, DeriveSelectChoice, AsFormField)]
enum Status {
    Active,     // id = "Active", name = "Active"
    Inactive,   // id = "Inactive", name = "Inactive"
    Pending,    // id = "Pending", name = "Pending"
}
```

## Benefits

1. **Massive reduction in boilerplate**: ~50 lines of manual trait implementation reduced to just derive attributes
2. **Less error-prone**: No risk of mismatched strings between `from_str`, `id`, and display methods
3. **Consistent behavior**: All enums using the derive macro will have identical `AsFormField` implementation
4. **Maintainable**: Changes to the enum variants automatically update all related methods

## Usage in Forms

The derived enums work seamlessly with Cot's form system:

```rust
#[derive(Form)]
struct TaskForm {
    title: String,
    priority: Priority,  // Uses our derived enum
    status: Status,      // Uses our simple enum
}
```

## Implementation Details

The `AsFormField` derive macro:
- Requires the enum to also derive `SelectChoice` (or implement it manually)
- Automatically implements `AsFormField` with `SelectField<Self>` as the field type
- Uses the `SelectChoice::from_str` method for validation in `clean_value`
- Uses the `SelectChoice::id` method for `to_field_value`
- Provides standard empty/required validation logic

This approach maintains full compatibility with existing code while providing a much more ergonomic developer experience for new enums.
