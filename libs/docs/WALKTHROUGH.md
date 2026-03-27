# RUSTY-ROLODEX WEEKLY WALKTHROUGH


## Rusty Rolodex - Week 8 Walkthrough
This week I created a Remote Storage that saves and load data from JSON endpoint (I used [jsonstorage.net](https://jsonstorage.net) for the implementation).

### Reqwest
I utilized the `reqwest::blocking` module to make http requests to save or retrieve data from the remote storage, blocking the current thread until the request fails or succeeds.

### Jsonstorage.net API
The fact that our ContactStore trait's core methods are `save()` and `load()` signals that for remote, well need a http GET request to load data and a POST request to save the data, jsonstorage.net offers an api-like json storage and it was the reason I chose the platform.

After acquiring an API key on the platform, I studied the api documentation:
- **Get JSON (can be public or require API key):** GET https://api.jsonstorage.net/v1/json/{userId}/{itemID}

- **Create JSON:** POST https://api.jsonstorage.net/v1/json?apiKey={your_api_key}  
A successful POST request returns a uri to the item it created, the uri contains the itemID in a GET request format.

- **Update JSON:** PUT https://api.jsonstorage.net/v1/json/{userId}/{itemID}?apiKey={your_api_key}

- **Patch JSON:** PATCH https://api.jsonstorage.net/v1/json/{userId}/{itemID}?apiKey={your_api_key}


### Implementation Detail
- The `RemoteStorage` struct has the following unique fields:
    + `base_url: Option<String>` holds the base url.
    + `resource_id: RefCell<Option<String>>` holds the itemID of our stored data for subsiquent update and retrieval.
    + `active_url: RefCell<Option<String>>` holds the formated url for the next request.

- The `RemoteStorage` struct uses the following method:
    + `RemoteStorage::format_get_req_from_base_url()` formats a GET request URL from the `base_url` and `resource_id` field, mutate the `active_url` with interior mutability, ready for the next GET request.
    + `RemoteStorage::format_post_req_from_base_url()` formats a POST request URL from the `base_url` field and API Key from .env, mutate the `active_url` with interior mutability, ready for the next POST request.
    + `RemoteStorage::format_put_req_from_base_url()` formats a PUT request URL from the `base_url` field, `resource_id` field, and API Key from .env, mutate the `active_url` with interior mutability, ready for the next PUT request.
    + `RemoteStorage::extract_resource_id_from_successful_post_req()` helps extract the itemID/resource_id from the URI returned from a successful POST request and set it .env config for subsequent retrieval.
    + `RemoteStorage::update_active_url_from_str()` directly updates `active_url` field from the provided url string. It is an alternative to be used if base_url, resource_id or any other data required to build a request url is not configured in .env. This method doesn't validate the url, hence we use the validate method before calling this method.

- **On save():**
    + On save operation the code will first attempt a PUT request if a resource_id is already configured in .env. This to avoid creating a new item on server each time we save, but instead update our already existing data with the resource_id.
    + If PUT request fails, the program **defaults to a POST request** to upload as a new item, then extract and configure the resource_id if successful.
    + If the save() operation fails, the error is propagated.

- **On load():**
    + The program uses the active_url to initiate a GET request.
    + The respose data is coverted to a string and parsed into `serde_json::from_str()` to create the contact_list HashMap.
    + Any error during the process is propagated.



### CAUTION!
This implementation is strictly based on [jsonstorage.net's](https://jsonstorage.net) API and might require some customization for other platform's API.  
Alternatively, you can provide an already formated GET request URL in the `--src` argument for the `Import` command. Likewise you can provide an already formated POST or PUT request URL in the `--des` arguement for `Export` command.



<!-- ### Demo week 4
![Demo GIF](./media/rolodex-demoV4.gif) -->




[project gist]: (https://gist.github.com/Iamdavidonuh/062da8918a2d333b2150c74cae6bd525)