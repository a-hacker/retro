#!/bin/bash

# Recreate config file
rm -rf ./env-config.js
touch ./env-config.js

cat >./env-config.js <<EOL
window._env_ = {
    REACT_APP_BACKEND_URI: "${REACT_APP_BACKEND_URI}",
    REACT_APP_WEBSOCKET_URI: "${REACT_APP_WEBSOCKET_URI}",
    REACT_APP_AUTH_URI: "${REACT_APP_AUTH_URI}",
}
EOL