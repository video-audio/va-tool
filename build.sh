#!/bin/bash
docker-compose -f ./dockerfiles/docker-compose.yml up --build && \
	docker cp dockerfiles_build-rs_1:/usr/local/bin/va-tool ./va-tool
