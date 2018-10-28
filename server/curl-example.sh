#!/bin/bash

PORT=62976

curl -d "{\"dir\":\"/Users/sadikovi/developer/omnisearch\",\"pattern\":\"os_supported\"}" -X POST http://127.0.0.1:$PORT/search

curl -d "{\"dir\":\"/Users/sadikovi/developer/omnisearch\"}" -X POST http://127.0.0.1:$PORT/cache/add

curl http://127.0.0.1:$PORT/cache/stats
