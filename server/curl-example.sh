#!/bin/bash

curl -d "{\"dir\":\"/Users/sadikovi/developer/spark\",\"pattern\":\"os_supported\"}" -X POST http://127.0.0.1:49555/search
curl -d "{\"dir\":\"/Users/sadikovi/developer/spark\",\"pattern\":\"os_supported\",\"use_cache\":true}" -X POST http://127.0.0.1:49555/search

curl -d "{\"dir\":\"/Users/sadikovi/developer/spark\"}" -X POST http://127.0.0.1:49555/cache/add

curl http://127.0.0.1:49555/cache/stats
