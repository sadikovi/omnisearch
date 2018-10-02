/** @babel */

export default {
  // Dispatches request to the server and invokes callback on a successful json.
  find(uri, dir, pattern, extensions, onSuccess, onError) {
    const data = {
      dir: dir,
      pattern: pattern,
      extensions: extensions
    };

    const options = {
      method: "POST",
      headers: {
        "Content-Type": "application/json; charset=utf-8"
      },
      body: JSON.stringify(data)
    };

    fetch(uri, options)
      .then(response => response.json())
      .then(json => onSuccess(json))
      .catch(err => onError(err))
  }
};
