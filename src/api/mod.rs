//! URL dispatcher for user account related API endpoints.

use jelly::actix_web::web::{delete, get, post, put, resource, scope, ServiceConfig};

pub mod services;

pub fn configure(config: &mut ServiceConfig) {
    config.service(
        scope("/api/v1")
            .service(
                resource("/post_package")
                    .route(post().to(services::package::controller::post_package)),
            )
            .service(
                resource("/search_package")
                    .route(post().to(services::package::controller::search_package)),
            )
            .service(
                resource("/download")
                    .route(post().to(services::package::controller::increase_download_count)),
            )
            .service(
                resource("/tokens/{token_id}")
                    .route(delete().to(services::setting::controllers::token::revoke_token)),
            )
            .service(
                resource("/tokens")
                    .route(put().to(services::setting::controllers::token::create_token)),
            )
            .service(
                resource("/me")
                    .route(get().to(services::setting::controllers::profile::get_logged_in_user)),
            )
            .service(
                scope("/packages/{package_name}")
                    .service(
                        resource("/badge")
                            .route(get().to(services::package::controller::package_badge_info)),
                    )
                    .service(
                        resource("/collaborators").route(
                            post().to(services::collaborators::controllers::add_collaborators),
                        ),
                    )
                    .service(resource("/transfer").route(
                        post().to(services::collaborators::controllers::transfer_ownership),
                    )),
            )
            .service(scope("/owner_invitations").service(
                resource("").route(post().to(services::collaborators::controllers::handle_invite)),
            )),
    );
}
