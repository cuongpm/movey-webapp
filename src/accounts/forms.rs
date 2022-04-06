use jelly::forms::{EmailField, PasswordField, Validation};
use serde::{Deserialize, Serialize};

fn default_redirect_path() -> String {
    "/".into()
}

#[derive(Default, Debug, Deserialize, Serialize)]
pub struct LoginForm {
    pub email: EmailField,
    pub password: PasswordField,

    #[serde(default = "default_redirect_path")]
    pub redirect: String,
}

impl Validation for LoginForm {
    fn is_valid(&mut self) -> bool {
        self.email.is_valid() && !self.password.value.is_empty()
    }
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct NewAccountForm {
    pub email: EmailField,
    pub password: PasswordField,
}

impl Validation for NewAccountForm {
    fn is_valid(&mut self) -> bool {
        self.email.is_valid()
            && self.password.is_valid()
            && self.password.validate_with(&[&self.email])
    }
}

#[derive(Default, Debug, Deserialize, Serialize)]
pub struct EmailForm {
    pub email: EmailField,
}

impl Validation for EmailForm {
    fn is_valid(&mut self) -> bool {
        self.email.is_valid()
    }
}

#[derive(Default, Debug, Deserialize, Serialize)]
pub struct ChangePasswordForm {
    // Unused in rendering, but stored here to enable password
    // checking with relative values.
    pub name: Option<String>,
    pub email: Option<String>,

    pub password: PasswordField,
    pub password_confirm: PasswordField,
}

impl Validation for ChangePasswordForm {
    fn is_valid(&mut self) -> bool {
        if !self.password.is_valid() || !self.password_confirm.is_valid() {
            return false;
        }

        if self.password.value != self.password_confirm.value {
            self.password
                .errors
                .push("Passwords must match.".to_string());
            return false;
        }

        self.password
            .validate_with(&[&self.name.as_ref().unwrap(), &self.email.as_ref().unwrap()])
    }
}

#[cfg(test)]
mod form_tests {
    mod new_account_form_tests {
        use super::super::*;
        #[test]
        fn is_valid_works() {
            let mut new_account_form = NewAccountForm {
                email: EmailField {
                    value: "valid@example.com".to_string(),
                    errors: vec![],
                },
                password: PasswordField {
                    value: "Strongpassword1@".to_string(),
                    errors: vec![],
                    hints: vec![],
                },
            };
            assert!(new_account_form.is_valid())
        }

        #[test]
        fn is_valid_with_short_password_return_false() {
            let mut new_account_form = NewAccountForm {
                email: EmailField {
                    value: "valid@example.com".to_string(),
                    errors: vec![],
                },
                password: PasswordField {
                    value: "12345".to_string(),
                    errors: vec![],
                    hints: vec![],
                },
            };
            new_account_form.is_valid();
            assert!(!new_account_form.is_valid())
        }
    }
}
