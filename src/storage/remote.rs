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
