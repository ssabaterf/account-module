use crate::{fairings::auth::AuthorizedUser, domain::user::Role};

pub fn can_continue(auth:AuthorizedUser, resource: &str) -> bool {
    if auth.role == Role::Admin {
        return true;
    }
    if auth.resource.contains(&resource.to_string()) {
        return true;
    }
    false
}

// pub fn can_continue_with_id(auth:AuthorizedUser, resource: &str, id: &str) -> bool {
//     if auth.role == Role::Admin {
//         return true;
//     }
//     if auth.resource.contains(&resource.to_string()) {
//         return true;
//     }
//     if auth.resource.contains(&format!("{}:{}", resource, id)) {
//         return true;
//     }
//     false
// }

pub fn only_admin(auth:AuthorizedUser) -> bool {
    if auth.role == Role::Admin {
        return true;
    }
    false
}