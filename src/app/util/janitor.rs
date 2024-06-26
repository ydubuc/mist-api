use std::{sync::Arc, time::Duration};

use tokio::{
    task,
    time::{interval, sleep},
};

use crate::{
    app::{self, util::time::current_time_in_secs},
    generate_media_requests::{
        self, dtos::get_generate_media_requests_filter_dto::GetGenerateMediaRequestsFilterDto,
        enums::generate_media_request_status::GenerateMediaRequestStatus,
    },
    media, AppState,
};

pub fn spawn(state: Arc<AppState>) {
    task::spawn(async move {
        sleep(Duration::from_secs(600)).await;
        let mut interval = interval(Duration::from_secs(600));

        loop {
            interval.tick().await;
            cleanup_requests(&state).await;
        }
    });
}

async fn cleanup_requests(state: &Arc<AppState>) {
    let ten_minutes_ago = (current_time_in_secs() as i64) - 600;
    let dto = GetGenerateMediaRequestsFilterDto {
        id: None,
        user_id: None,
        status: Some(GenerateMediaRequestStatus::Processing.value().to_string()),
        api_v: Some(app::config::API_V),
        sort: Some("created_at,asc".to_string()),
        cursor: None,
        limit: None,
    };

    match generate_media_requests::service::get_generate_media_requests_as_admin(&dto, &state.pool)
        .await
    {
        Ok(requests) => {
            if requests.len() > 0 {
                tracing::debug!("received {} request(s) to clean up", requests.len());
            }

            let mut futures = Vec::new();

            for request in &requests {
                if request.created_at > ten_minutes_ago {
                    continue;
                }

                futures.push(media::service::on_generate_media_completion_with_retry(
                    request,
                    &GenerateMediaRequestStatus::Error,
                    &None,
                    &state,
                ));
            }

            let _ = futures::future::join_all(futures).await;
        }
        Err(e) => {
            tracing::error!("cleanup_requests: {:?}", e);
        }
    }
}
