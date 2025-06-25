VOLUME=$(shell realpath `pwd`/..)

enter:
	docker run -it --rm --env-file=.env \
		-v ${VOLUME}:/data qnx-ubuntu

build:
	. ./.env; docker-buildx build \
		--build-arg MYQNX_LICENSE=$$MYQNX_LICENSE \
		--build-arg MYQNX_USER=$$MYQNX_USER \
		--build-arg MYQNX_PASSWORD=$$MYQNX_PASSWORD \
		-t qnx-ubuntu .
