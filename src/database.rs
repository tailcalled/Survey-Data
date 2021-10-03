use std::collections::HashMap;
use serde_json::{from_value, json, Value};
use sqlx::Postgres;
use sqlx::pool::PoolConnection;
use uuid::Uuid;

pub async fn get_or_create_response(response_id: Uuid, conn: &mut PoolConnection<Postgres>) -> HashMap<String, Value> {
	let res = sqlx::query!(
		"SELECT response_id, user_id, submit_time, content FROM responses WHERE response_id = $1",
		response_id
	).fetch_optional(&mut*conn).await.unwrap();
	match res {
		Some(it) => {
			let map = it.content;
			from_value(map).unwrap()
		}
		None => {
			sqlx::query!(
				"INSERT INTO responses(response_id, user_id, submit_time, content)\
				              VALUES($1, $2, NOW(), $3)",
				response_id, Option::<Uuid>::None, json!({})
			).execute(&mut*conn).await.unwrap();
			HashMap::new()
		}
	}
}

pub async fn update_response(response_id: Uuid, resp_map: HashMap<String, Value>, conn: &mut PoolConnection<Postgres>) {
	let mut prev_map = get_or_create_response(response_id, conn).await;
	for kv in resp_map {
		prev_map.insert(kv.0, kv.1);
	}
    sqlx::query!(
		"UPDATE responses SET content = $2, submit_time = NOW() WHERE response_id = $1",
		response_id, serde_json::to_value(&prev_map).unwrap()
	).execute(&mut*conn).await.unwrap();
}

pub async fn get_all_responses(conn: &mut PoolConnection<Postgres>) -> String {
	let res = sqlx::query!(
		"SELECT response_id, user_id, submit_time, content FROM  responses"
	).fetch_all(&mut*conn).await.unwrap();
	format!("{:#?}", res).into()
}