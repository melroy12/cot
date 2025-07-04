use cot::db::{Auto, ForeignKey};
use cot::form::fields::SelectChoice;
use cot::form::{
    AsFormField, Form, FormContext, FormErrorTarget, FormField, FormFieldValidationError,
    FormResult,
};
use cot::test::TestRequestBuilder;
use cot_macros::SelectChoice as DeriveSelectChoice;
use cot_macros::{AsFormField, model};

#[derive(Debug, Form)]
struct MyForm {
    name: String,
    address: Option<String>,
    age: u8,
}

#[cot::test]
async fn context_from_empty_request() {
    let mut request = TestRequestBuilder::get("/").build();

    let context = MyForm::build_context(&mut request).await;
    assert!(context.is_ok());
}

#[cot::test]
async fn context_display_non_empty() {
    let mut request = TestRequestBuilder::get("/").build();

    let context = MyForm::build_context(&mut request).await.unwrap();
    let form_rendered = context.to_string();
    assert!(!form_rendered.is_empty());
}

#[cot::test]
async fn form_from_request() {
    let mut request = TestRequestBuilder::post("/")
        .form_data(&[("name", "Alice"), ("age", "30")])
        .build();

    let form = MyForm::from_request(&mut request).await.unwrap().unwrap();
    assert_eq!(form.name, "Alice");
    assert_eq!(form.address, None);
    assert_eq!(form.age, 30);
}

#[cot::test]
async fn form_errors_required() {
    let mut request = TestRequestBuilder::post("/")
        .form_data::<String>(&[])
        .build();

    let form = MyForm::from_request(&mut request).await;
    match form {
        Ok(FormResult::ValidationError(context)) => {
            assert_eq!(context.errors_for(FormErrorTarget::Form), &[]);
            assert_eq!(
                context.errors_for(FormErrorTarget::Field("name")),
                &[FormFieldValidationError::Required]
            );
            assert_eq!(context.errors_for(FormErrorTarget::Field("address")), &[]);
            assert_eq!(
                context.errors_for(FormErrorTarget::Field("age")),
                &[FormFieldValidationError::Required]
            );
        }
        _ => panic!("Expected a validation error"),
    }
}

#[cot::test]
async fn values_persist_on_form_errors() {
    let mut request = TestRequestBuilder::post("/")
        .form_data(&[("name", "Alice"), ("age", "invalid")])
        .build();

    let form = MyForm::from_request(&mut request).await;
    match form {
        Ok(FormResult::ValidationError(context)) => {
            assert_eq!(context.name.value(), Some("Alice"));
            assert_eq!(context.age.value(), Some("invalid"));

            assert_eq!(context.errors_for(FormErrorTarget::Form), &[]);
            assert_eq!(context.errors_for(FormErrorTarget::Field("name")), &[]);
            assert_eq!(context.errors_for(FormErrorTarget::Field("address")), &[]);
            assert_eq!(
                context.errors_for(FormErrorTarget::Field("age")),
                &[FormFieldValidationError::InvalidValue(
                    "invalid".to_string()
                )]
            );
        }
        _ => panic!("Expected a validation error"),
    }
}

#[cot::test]
async fn foreign_key_field() {
    #[model]
    struct TestModel {
        #[model(primary_key)]
        name: String,
    }

    #[derive(Form)]
    struct TestModelForm {
        test_field: ForeignKey<TestModel>,
    }

    // test field rendering
    let context = TestModelForm::build_context(&mut TestRequestBuilder::get("/").build())
        .await
        .unwrap();
    let form_rendered = context.to_string();
    assert!(form_rendered.contains("test_field"));
    assert!(form_rendered.contains("type=\"text\""));

    // test form data
    let mut request = TestRequestBuilder::post("/")
        .form_data(&[("test_field", "Alice")])
        .build();
    let form = TestModelForm::from_request(&mut request).await;
    match form {
        Ok(FormResult::Ok(instance)) => {
            assert_eq!(instance.test_field.primary_key(), "Alice");
        }
        _ => panic!("Expected a valid form"),
    }

    // test re-raising validation errors
    let mut request = TestRequestBuilder::post("/")
        .form_data(&[("test_field", "")])
        .build();
    let form = TestModelForm::from_request(&mut request).await;
    match form {
        Ok(FormResult::ValidationError(context)) => {
            assert_eq!(
                context.errors_for(FormErrorTarget::Field("test_field")),
                &[FormFieldValidationError::Required]
            );
        }
        _ => panic!("Expected a validation error"),
    }
}

#[cot::test]
async fn foreign_key_field_to_field_value() {
    #[model]
    struct TestModel {
        #[model(primary_key)]
        id: Auto<i32>,
    }

    let field_value = ForeignKey::<TestModel>::Model(Box::new(TestModel {
        id: Auto::fixed(123),
    }))
    .to_field_value();
    assert_eq!(field_value, "123");

    let field_value = ForeignKey::<TestModel>::PrimaryKey(Auto::fixed(456)).to_field_value();
    assert_eq!(field_value, "456");
}

#[derive(Debug, Clone, PartialEq, DeriveSelectChoice, AsFormField)]
enum Priority {
    #[select_choice(id = "low", name = "Low Priority")]
    Low,
    #[select_choice(id = "medium", name = "Medium Priority")]
    Medium,
    #[select_choice(id = "high", name = "High Priority")]
    High,
}

#[derive(Debug, Form)]
struct SimpleTaskForm {
    title: String,
    priority: Priority,
}

#[cot::test]
async fn select_field_form_integration() {
    let mut request = TestRequestBuilder::post("/")
        .form_data(&[("title", "Complete project"), ("priority", "high")])
        .build();

    let form = SimpleTaskForm::from_request(&mut request)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(form.title, "Complete project");
    assert_eq!(form.priority, Priority::High);
}

#[cot::test]
async fn select_field_validation_error() {
    let mut request = TestRequestBuilder::post("/")
        .form_data(&[("title", "Test task"), ("priority", "invalid_priority")])
        .build();

    let form = SimpleTaskForm::from_request(&mut request).await;
    match form {
        Ok(FormResult::ValidationError(context)) => {
            assert_eq!(context.errors_for(FormErrorTarget::Form), &[]);
            assert_eq!(context.errors_for(FormErrorTarget::Field("title")), &[]);
            assert_eq!(
                context.errors_for(FormErrorTarget::Field("priority")),
                &[FormFieldValidationError::InvalidValue(
                    "invalid_priority".to_string()
                )]
            );
        }
        _ => panic!("Expected a validation error"),
    }
}

#[cot::test]
async fn select_field_context_display() {
    let mut request = TestRequestBuilder::get("/").build();

    let context = SimpleTaskForm::build_context(&mut request).await.unwrap();
    let form_rendered = context.to_string();

    assert!(form_rendered.contains("<select"));
    assert!(form_rendered.contains("name=\"priority\""));
    assert!(form_rendered.contains("Low Priority"));
    assert!(form_rendered.contains("Medium Priority"));
    assert!(form_rendered.contains("High Priority"));
    assert!(form_rendered.contains("value=\"low\""));
    assert!(form_rendered.contains("value=\"medium\""));
    assert!(form_rendered.contains("value=\"high\""));
}

#[cot::test]
async fn derive_macro_comparison() {
    // Test that our derived Priority enum works exactly the same as manual implementation

    // Test basic functionality
    assert_eq!(Priority::Low.id(), "low");
    assert_eq!(Priority::Medium.id(), "medium");
    assert_eq!(Priority::High.id(), "high");

    assert_eq!(Priority::Low.to_string(), "Low Priority");
    assert_eq!(Priority::Medium.to_string(), "Medium Priority");
    assert_eq!(Priority::High.to_string(), "High Priority");

    // Test from_str
    assert_eq!(Priority::from_str("low").unwrap(), Priority::Low);
    assert_eq!(Priority::from_str("medium").unwrap(), Priority::Medium);
    assert_eq!(Priority::from_str("high").unwrap(), Priority::High);
    assert!(Priority::from_str("invalid").is_err());

    // Test default_choices
    let choices = Priority::default_choices();
    assert_eq!(choices.len(), 3);
    assert!(choices.contains(&Priority::Low));
    assert!(choices.contains(&Priority::Medium));
    assert!(choices.contains(&Priority::High));

    // Test AsFormField integration
    assert_eq!(Priority::High.to_field_value(), "high");
}

// Example of a simpler enum that uses defaults
#[derive(Debug, Clone, PartialEq, DeriveSelectChoice, AsFormField)]
enum Status {
    Active,
    Inactive,
    Pending,
}

#[cot::test]
async fn simple_enum_with_defaults() {
    // When no custom id/name is specified, it should use the variant name
    assert_eq!(Status::Active.id(), "Active");
    assert_eq!(Status::Inactive.id(), "Inactive");
    assert_eq!(Status::Pending.id(), "Pending");

    assert_eq!(Status::Active.to_string(), "Active");
    assert_eq!(Status::Inactive.to_string(), "Inactive");
    assert_eq!(Status::Pending.to_string(), "Pending");

    assert_eq!(Status::from_str("Active").unwrap(), Status::Active);
    assert_eq!(Status::from_str("Inactive").unwrap(), Status::Inactive);
    assert_eq!(Status::from_str("Pending").unwrap(), Status::Pending);
}
