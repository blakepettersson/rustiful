use super::iron::*;
use super::iron::Headers;
use super::iron::headers::ContentType;
use super::iron::mime::Mime;
use super::iron_test::{request, response};
use r2d2::GetTimeout;
use resources::diesel_resource::*;
use rustiful::*;
use rustiful::iron::*;
use rustiful::iron::status::Status;
use serde_json;
use std::fmt::Display;
use uuid::Uuid;

impl FromRequest for DB {
    type Error = GetTimeout;

    fn from_request(_: &Request) -> Result<DB, Self::Error> {
        match DB_POOL.get() {
            Ok(conn) => Ok(DB(conn)),
            Err(e) => Err(e),
        }
    }
}

impl<'a> From<&'a MyErr> for Status {
    fn from(err: &'a MyErr) -> Self {
        match *err {
            MyErr::UpdateError(_) => Status::ImATeapot,
            _ => Status::InternalServerError,
        }
    }
}

fn app_router() -> Chain {
    let mut router = JsonApiRouterBuilder::default();
    router.jsonapi_get::<Test>();
    router.jsonapi_post::<Test>();
    router.jsonapi_index::<Test>();
    router.jsonapi_patch::<Test>();
    router.jsonapi_delete::<Test>();
    router.build()
}

#[test]
#[ignore] // Ignored by default since we need to run this sequentially, due to SQLite locking.
fn post_without_client_generated_id() {
    let data = r#"
    {
        "data": {
            "type": "tests",
            "attributes": {
                "body": "test",
                "title": "test",
                "published": true
            }
        }
    }"#;

    let created = do_post(&data);
    let retrieved = do_get(&created.clone().data.id.unwrap());

    assert_eq!(created, retrieved);
}

#[test]
#[ignore] // Ignored by default since we need to run this sequentially, due to SQLite locking.
fn post_with_client_generated_id() {
    let id = Uuid::new_v4().to_string();
    let data = format!(
        r#"
    {{
        "data": {{
            "id": "{}",
            "type": "tests",
            "attributes": {{
                "body": "test",
                "title": "test",
                "published": true
            }}
        }}
    }}"#,
        id
    );

    let created = do_post(&data);
    let retrieved = do_get(&id);

    assert_eq!(created, retrieved);
}

#[test]
#[ignore] // Ignored by default since we need to run this sequentially, due to SQLite locking.
fn post_with_client_generated_id_and_fieldset_params() {
    let id = Uuid::new_v4().to_string();
    let data = format!(
        r#"
    {{
        "data": {{
            "id": "{}",
            "type": "tests",
            "attributes": {{
                "body": "test",
                "title": "test",
                "published": true
            }}
        }}
    }}"#,
        id
    );

    let created = do_post_with_url(&data, "http://localhost:3000/tests?fields[tests]=title");
    let expected = JsonApiData::new(
        Some(id),
        "tests",
        <Test as ToJson>::Attrs::new(Some("test".to_string()), Some(None), None),
    );

    assert_eq!(created.data, expected);
}

#[test]
#[ignore] // Ignored by default since we need to run this sequentially, due to SQLite locking.
fn update_with_nullable_field() {
    let id = Uuid::new_v4().to_string();
    let data = format!(
        r#"
    {{
        "data": {{
            "id": "{}",
            "type": "tests",
            "attributes": {{
                "body": "test",
                "title": "test",
                "published": true
            }}
        }}
    }}"#,
        &id
    );

    do_post(&data);

    {
        let patch = format!(
            r#"
        {{
            "data": {{
                "id": "{}",
                "type": "tests",
                "attributes": {{
                    "title": "funky"
                }}
            }}
        }}"#,
            &id
        );

        do_patch(&id, &patch);


        let retrieved = do_get(&id);
        assert_eq!(
            Some("test".to_string()),
            retrieved.data.attributes.body.unwrap()
        );
        assert_eq!(Some("funky".to_string()), retrieved.data.attributes.title);
    }

    {
        let patch = format!(
            r#"
        {{
            "data": {{
                "id": "{}",
                "type": "tests",
                "attributes": {{
                    "body": "new_content"
                }}
            }}
        }}"#,
            &id
        );

        do_patch(&id, &patch);

        let retrieved = do_get(&id);
        assert_eq!(
            Some("new_content".to_string()),
            retrieved.data.attributes.body.unwrap()
        );
    }

    {
        let patch = format!(
            r#"
        {{
            "data": {{
                "id": "{}",
                "type": "tests",
                "attributes": {{
                    "body": null
                }}
            }}
        }}"#,
            &id
        );

        do_patch(&id, &patch);

        let retrieved = do_get(&id);
        assert_eq!(None, retrieved.data.attributes.body.unwrap());
    }
}

#[test]
#[ignore] // Ignored by default since we need to run this sequentially, due to SQLite locking.
fn update_with_fieldset() {
    let id = Uuid::new_v4().to_string();
    let data = format!(
        r#"
    {{
        "data": {{
            "id": "{}",
            "type": "tests",
            "attributes": {{
                "body": "test",
                "title": "test",
                "published": true
            }}
        }}
    }}"#,
        &id
    );

    do_post(&data);

    {
        let patch = format!(
            r#"
        {{
            "data": {{
                "id": "{}",
                "type": "tests",
                "attributes": {{
                    "title": "funky"
                }}
            }}
        }}"#,
            &id
        );

        let updated = do_patch_with_url(&id, &patch, "fields[tests]=title");
        let expected = JsonApiData::new(
            Some(id),
            "tests",
            <Test as ToJson>::Attrs::new(Some("funky".to_string()), Some(None), None),
        );

        assert_eq!(updated.data, expected);
    }
}

fn do_get<T: Display>(id: &T) -> JsonApiContainer<JsonApiData<Test>> {
    let response = request::get(
        &format!("http://localhost:3000/tests/{}", id),
        Headers::new(),
        &app_router(),
    );
    let result = response::extract_body_to_string(response.unwrap());
    serde_json::from_str(&result).unwrap()
}

fn do_post(json: &str) -> JsonApiContainer<JsonApiData<Test>> {
    do_post_with_url(json, "http://localhost:3000/tests")
}

fn do_post_with_url(json: &str, url: &str) -> JsonApiContainer<JsonApiData<Test>> {
    let content_type: Mime = "application/vnd.api+json".parse().unwrap();

    let mut headers = Headers::new();
    headers.set::<ContentType>(ContentType(content_type));

    let response = request::post(url, headers, &json, &app_router());
    let result = response::extract_body_to_string(response.unwrap());

    serde_json::from_str(&result).unwrap()
}

fn do_patch<T: Display>(id: &T, json: &str) -> JsonApiContainer<JsonApiData<Test>> {
    do_patch_with_url(id, json, "")
}

fn do_patch_with_url<T: Display>(
    id: &T,
    json: &str,
    query: &str,
) -> JsonApiContainer<JsonApiData<Test>> {
    let content_type: Mime = "application/vnd.api+json".parse().unwrap();

    let mut headers = Headers::new();
    headers.set::<ContentType>(ContentType(content_type));

    let response = request::patch(
        &format!("http://localhost:3000/tests/{}?{}", id, query),
        headers,
        &json,
        &app_router(),
    );
    let result = response::extract_body_to_string(response.unwrap());
    serde_json::from_str(&result).unwrap()
}
