use crate::helper;
use std::cell::RefCell;

use super::{AppError, Contact, ContactStore, HashMap, Uuid};
use reqwest::blocking;
use url::Url;

pub struct RemoteStorage {
    pub medium: String,
    pub base_url: Option<String>,
    resource_id: RefCell<Option<String>>,
    pub active_url: RefCell<Option<String>>,
}

impl RemoteStorage {
    pub fn new() -> Result<Self, AppError> {
        Ok(Self {
            medium: "remote".to_string(),
            base_url: helper::get_env_value_by_key("REMOTE_STORAGE_URL").ok(),
            resource_id: RefCell::new(helper::get_env_value_by_key("RESOURCE_ID").ok()),
            active_url: RefCell::new(None),
        })
    }

    /// This method formats a get-request url for the remote storage
    /// from the predefined base url in .env and uses RefCell interior mutability
    /// to update the `active_url` field with the formated url for the next request.
    ///
    /// Use `Self.update_active_url_from_str()` method as an alternative to explicitly
    /// parse a url as string if .env is not set.
    pub fn format_get_req_from_base_url(&self) -> Result<(), AppError> {
        let mut resource_id = self.resource_id.borrow_mut();

        if resource_id.is_none() {
            // if resource_id is not set, try read from .env
            let id_from_env = helper::get_env_value_by_key("RESOURCE_ID").map_or_else(
                |_| Err(AppError::NotFound("Resource ID in env".to_string())),
                Ok,
            )?; // Propagate error to exit Fn if resource_id is not set in .env

            *resource_id = Some(id_from_env);
        }

        if let Some(base_url) = &self.base_url {
            *self.active_url.borrow_mut() =
                Some(format!("{}/{}", base_url, resource_id.as_ref().unwrap()));

            Ok(())
        } else {
            Err(AppError::NotFound("Base URL.".to_string()))
        }
    }

    /// This method formats a post-request url that incorperates the API key
    /// for the remote storage from the predefined base url in .env and uses RefCell
    /// interior mutability to update the `active_url` field with the formated url for
    /// the next request.
    ///
    /// Use `Self.update_active_url_from_str()` method as an alternative to explicitly
    /// parse a url as string if .env is not set.
    pub fn format_post_req_from_base_url(&self) -> Result<(), AppError> {
        if let Some(base_url) = &self.base_url {
            *self.active_url.borrow_mut() = Some(format!(
                "{}?apiKey={}",
                base_url,
                helper::get_env_value_by_key("REMOTE_API_KEY")?
            ));

            Ok(())
        } else {
            Err(AppError::NotFound("Base URL.".to_string()))
        }
    }

    /// This method formats a put-request url for the remote storage
    /// from the predefined base url in .env and uses RefCell interior mutability
    /// to update the `active_url` field with the formated url for the next request.
    ///
    /// Use `Self.update_active_url_from_str()` method as an alternative to explicitly
    /// parse a url as string if .env is not set.
    pub fn format_put_req_from_base_url(&self) -> Result<(), AppError> {
        let mut resource_id = self.resource_id.borrow_mut();

        if resource_id.is_none() {
            // if resource_id is not set, try read from .env
            let id_from_env = helper::get_env_value_by_key("RESOURCE_ID").map_or_else(
                |_| Err(AppError::NotFound("Resource ID in env".to_string())),
                Ok,
            );

            // If resource id not found then just use post format to upload fresh data (not update existing)
            if id_from_env.is_err() {
                return self.format_post_req_from_base_url();
            }
            *resource_id = Some(id_from_env.unwrap());
        }

        if let Some(base_url) = &self.base_url {
            *self.active_url.borrow_mut() = Some(format!(
                "{}/{}?apiKey={}",
                base_url,
                resource_id.as_ref().unwrap(),
                helper::get_env_value_by_key("REMOTE_API_KEY")?
            ));

            Ok(())
        } else {
            Err(AppError::NotFound("Base URL.".to_string()))
        }
    }

    /// This function extracts the resource id from the `uri` arguement based on the
    /// url pattern documented in https://app.jsonstorage.net, which was used during
    /// the development of this project
    pub fn extract_resource_id_from_successful_post_req(&self, uri: Option<&String>) {
        if uri.is_none() {
            return;
        }

        let uri = uri.unwrap();
        let uri_parts: Vec<&str> = uri.split("json/").collect();
        let resource_id = uri_parts[uri_parts.len() - 1].to_string();

        let _ = helper::set_env_value_in_file("RESOURCE_ID", &resource_id);

        *self.resource_id.borrow_mut() = Some(resource_id);
    }

    /// This method updates the `active_url` field directly from string arguement.
    /// It can be used as an alternative when url is not set in .env.
    ///
    /// ## Caution!
    /// This method **does not validate/verify url.** Validate url before
    /// parsing to method.
    pub fn update_active_url_from_str(&self, uri: &str) {
        *self.active_url.borrow_mut() = Some(uri.to_string());
    }
}

impl ContactStore for RemoteStorage {
    fn get_medium(&self) -> &str {
        &self.medium
    }

    fn load(&self) -> Result<HashMap<Uuid, Contact>, AppError> {
        let active_uri = self.active_url.borrow().clone();
        let active_uri = active_uri.ok_or(AppError::NotFound("active url".to_string()))?;

        let url = active_uri;
        let response = blocking::get(url)?;

        let response = response.error_for_status()?;
        let res_str = response.text()?;
        let contacts: HashMap<Uuid, Contact> = serde_json::from_str(&res_str)?;
        Ok(contacts)
    }

    fn save(&self, contacts: &HashMap<Uuid, Contact>) -> Result<(), AppError> {
        let url = self.active_url.borrow().clone();
        let url = url.ok_or(AppError::NotFound("active url".to_string()))?;

        let blocking_client = blocking::Client::new();
        let mut res = blocking_client
            .put(&url)
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .body(serde_json::to_vec(contacts)?)
            .send()?;

        if !res.status().is_success() {
            res = blocking_client
                .post(&url)
                .header(reqwest::header::CONTENT_TYPE, "application/json")
                .body(serde_json::to_vec(contacts)?)
                .send()?;
        }

        // Convert non-success status into a `reqwest::Error` which maps to `AppError::FailedRequest`
        let res = res.error_for_status()?;

        let res_map: HashMap<String, String> = serde_json::from_str(&res.text()?)?;
        self.extract_resource_id_from_successful_post_req(res_map.get("uri"));
        Ok(())
    }
}

pub fn is_valid_url(url: &str) -> bool {
    Url::parse(url).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito::{mock, server_url};

    // Implementing a MockRemoteStorage to strip out
    // all calls to read .env value.
    struct MockRemoteStorage {
        pub medium: String,
        pub base_url: Option<String>,
        resource_id: RefCell<Option<String>>,
        pub active_url: RefCell<Option<String>>,
    }

    impl MockRemoteStorage {
        fn format_get_req_from_base_url(&self) -> Result<(), AppError> {
            let resource_id = self.resource_id.borrow_mut();

            if let Some(base_url) = &self.base_url {
                *self.active_url.borrow_mut() =
                    Some(format!("{}/{}", base_url, resource_id.as_ref().unwrap()));

                Ok(())
            } else {
                Err(AppError::NotFound("Base URL.".to_string()))
            }
        }

        fn format_put_req_from_base_url(&self) -> Result<(), AppError> {
            let resource_id: std::cell::RefMut<'_, Option<String>> = self.resource_id.borrow_mut();

            if let Some(base_url) = &self.base_url {
                *self.active_url.borrow_mut() =
                    Some(format!("{}/{}", base_url, resource_id.as_ref().unwrap(),));

                Ok(())
            } else {
                Err(AppError::NotFound("Base URL.".to_string()))
            }
        }
    }

    impl ContactStore for MockRemoteStorage {
        fn get_medium(&self) -> &str {
            &self.medium
        }

        fn load(&self) -> Result<HashMap<Uuid, Contact>, AppError> {
            let active_uri = self.active_url.borrow().clone();
            let active_uri = active_uri.ok_or(AppError::NotFound("active url".to_string()))?;

            let url = active_uri;
            let response = blocking::get(url)?;

            let response = response.error_for_status()?;
            let res_str = response.text()?;
            let contacts: HashMap<Uuid, Contact> = serde_json::from_str(&res_str)?;
            Ok(contacts)
        }

        fn save(&self, contacts: &HashMap<Uuid, Contact>) -> Result<(), AppError> {
            let url = self.active_url.borrow().clone();
            let url = url.ok_or(AppError::NotFound("active url".to_string()))?;

            let blocking_client = blocking::Client::new();
            let mut res = blocking_client
                .put(&url)
                .header(reqwest::header::CONTENT_TYPE, "application/json")
                .body(serde_json::to_vec(contacts)?)
                .send()?;

            if !res.status().is_success() {
                res = blocking_client
                    .post(&url)
                    .header(reqwest::header::CONTENT_TYPE, "application/json")
                    .body(serde_json::to_vec(contacts)?)
                    .send()?;
            }

            // Convert non-success status into a `reqwest::Error` which maps to `AppError::FailedRequest`
            let res = res.error_for_status()?;

            let _res_map: HashMap<String, String> = serde_json::from_str(&res.text()?)?;
            Ok(())
        }
    }

    #[test]
    fn load_fetches_contacts_from_remote() {
        // JSON mapping of uuid -> Contact (must match your Contact serde shape)
        let contacts_json = r#"
        {
            "ed70c65e-a25d-4c00-9633-f6bae773989d":{
                "id":"ed70c65e-a25d-4c00-9633-f6bae773989d",
                "name":"Lauren",
                "phone":"09159652486",
                "email":"yangbrandon@gmail.com",
                "tag":"family",
                "deleted":false,
                "created_at":"2025-12-08T14:08:47.112315605Z",
                "updated_at":"2026-02-07T18:33:31.733469575Z"
            },
            "12937456-c347-492f-a0d8-9fd2efd35eb0":{
                "id":"12937456-c347-492f-a0d8-9fd2efd35eb0",
                "name":"Adamu",
                "phone":"07254715016",
                "email":"lindalewis@gmail.com",
                "tag":"others",
                "deleted":false,
                "created_at":"2025-12-08T14:08:47.112315605Z",
                "updated_at":"2026-02-07T18:27:37.993254387Z"
            },
            "06b2f175-3833-4df3-b834-f8a042ff4736":{
                "id":"06b2f175-3833-4df3-b834-f8a042ff4736",
                "name":"Kathy",
                "phone":"09150505086",
                "email":"quinnjennifer@gmail.com",
                "tag":"others",
                "deleted":false,
                "created_at":"2025-12-08T14:08:47.112315605Z",
                "updated_at":"2026-02-07T18:36:14.497210882Z"
            }
        }
        "#;

        // Setup mock for GET /resource-id
        let _m = mock("GET", "/resource-id")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(contacts_json)
            .create();

        // Construct the storage and point it at the mock server
        let storage = MockRemoteStorage {
            medium: "remote".to_string(),
            base_url: Some(server_url()), // server_url() is like "http://127.0.0.1:XXXXX"
            resource_id: std::cell::RefCell::new(Some("resource-id".to_string())),
            active_url: std::cell::RefCell::new(None),
        };

        // Make sure active_url is set to the mock endpoint
        storage.format_get_req_from_base_url().unwrap();
        let contacts = storage.load().unwrap();

        assert_eq!(contacts.len(), 3);
        assert!(contacts.values().any(|c| c.name == "Adamu"));
    }

    #[test]
    fn save_prefers_put_then_post_when_put_fails() {
        let contacts_json = r#"
        {
            "ed70c65e-a25d-4c00-9633-f6bae773989d":{
                "id":"ed70c65e-a25d-4c00-9633-f6bae773989d",
                "name":"Lauren",
                "phone":"09159652486",
                "email":"yangbrandon@gmail.com",
                "tag":"family",
                "deleted":false,
                "created_at":"2025-12-08T14:08:47.112315605Z",
                "updated_at":"2026-02-07T18:33:31.733469575Z"
            },
            "12937456-c347-492f-a0d8-9fd2efd35eb0":{
                "id":"12937456-c347-492f-a0d8-9fd2efd35eb0",
                "name":"Adamu",
                "phone":"07254715016",
                "email":"lindalewis@gmail.com",
                "tag":"others",
                "deleted":false,
                "created_at":"2025-12-08T14:08:47.112315605Z",
                "updated_at":"2026-02-07T18:27:37.993254387Z"
            },
            "06b2f175-3833-4df3-b834-f8a042ff4736":{
                "id":"06b2f175-3833-4df3-b834-f8a042ff4736",
                "name":"Kathy",
                "phone":"09150505086",
                "email":"quinnjennifer@gmail.com",
                "tag":"others",
                "deleted":false,
                "created_at":"2025-12-08T14:08:47.112315605Z",
                "updated_at":"2026-02-07T18:36:14.497210882Z"
            }
        }
        "#;

        // construct a small contacts map with serde-serializable data matching your types.
        let contacts: HashMap<Uuid, Contact> = serde_json::from_str(&contacts_json).unwrap();

        // Return 500 for PUT, then 200 for POST with small body.
        let put_mock = mock("PUT", "/resource-id").with_status(500).create();

        let post_mock = mock("POST", "/resource-id")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"success":"true"}"#)
            .create();

        let storage = MockRemoteStorage {
            medium: "remote".to_string(),
            base_url: Some(server_url()),
            resource_id: std::cell::RefCell::new(Some("resource-id".to_string())),
            active_url: std::cell::RefCell::new(None),
        };

        storage.format_put_req_from_base_url().unwrap();

        // Call save; code will attempt PUT, then POST on non-success, then parse response.
        let res = storage.save(&contacts);
        assert!(res.is_ok());

        put_mock.assert();
        post_mock.assert();
    }
}
