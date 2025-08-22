---
date: 08/22/2025
title: Setting up Internal Docker Registries
blurb: Learn how to set up Private docker registries for internal lab use
---
Registries are a way to save and share Docker images with other users. The defacto way to save these registries is through Docker Hub, where you can save and share registries with the public. However  are also other container registries such as Aamazon Elastic Container Registry, Google Container Registry, and more. All of these options host open source and public images, and even allow for private registries. Unfortunately these private registries can be limited, and there's a use case to have a private registry running on an internal network to save and share Docker images.


This article shows how to create a private Docker registry amongst for an internal development lab environment.
### Steps
#### Configure Private Docker Registry

To run a private registry, we set up a registry by running a container of a registry image.

To be able to allow http (for development only) requests to the registry we have to add
````
{
    "insecure-registries":[
        "localhost:5400"
    ]
}
````

to `/etc/docker/daemon.json` (create it if it doesn’t exist). Then `systemctl daemon-reload` and restart docker.
#### Run Docker Registry

Docker provides a  [public image](https://hub.docker.com/_/registry)  of a registry. We’ll have to pull the registry from docker hub.

`docker pull registry`

You’ll see that the registry exists docker images under registry.
````
mav@sgvmcloud:~/docker_learning/helloworld-demo-node$ docker images
REPOSITORY                         TAG       IMAGE ID       CREATED          SIZE
localhost:5400/docker-quickstart   1.0       72432157603b   22 minutes ago   142MB
hello-world                        latest    1b44b5a3e06a   2 weeks ago      10.1kB
traefik                            v3.4      aa215d9f973d   4 weeks ago      227MB
mysql                              9.3       850100bac3be   4 months ago     859MB
registry                           latest    3c52eedeec80   4 months ago     57.7MB
phpmyadmin                         latest    21c6d797c79c   7 months ago     568MB
````

Now run the registry image locally. 
```
run -d -p 5400:5000 --name pi-registry registry
```
Make sure the port 5400 corresponds to the insecure-registries port value.

> It’s important to have the endpoint to be 5000 in this example.

`docker ps` to verify it exists.

You’ll see that there’s a registry endpoint that’s proxied through port 5400 to port 5000
```
    CONTAINER ID   IMAGE      COMMAND                  CREATED             STATUS             PORTS                                         NAMES
    8c32ab8bfff4   registry   "/entrypoint.sh /etc…"   About an hour ago   Up About an hour   0.0.0.0:5400->5000/tcp, [::]:5400->5000/tcp   pi-registry
```

> If you did configure the insecure registries, on the this step you’ll get an error because docker will default to https.
```
mav@sgvmcloud:~/docker_learning/helloworld-demo-node$ docker run -p 5400:5000 registry
.
.
.
time="2025-08-22T18:51:47.037943018Z" level=error msg="traces export: Post \"https://localhost:4318/v1/traces\": dial tcp [::1]:4318: connect: connection refused" environment=development go.version=go1.23.7 instance.id=fdef731f-0a0f-4a63-a2de-aa9ba2512de0 service=registry version=3.0.0
```

Verify that the registry is up by trying to list the repositories on that registry container 
```
curl http://localhost:5400/v2/_catalog
{"repositories":[]}
```
#### Push Images to Registry

We’re now going to use the registry by giving an example. The TLDR is

- After creating your image tag them with the url where the local registry sits. 
```docker tag some-docker-image localhost:5400/some-docker-image:1.0```

- `docker push localhost:5400/some-docker-image:1.0`

 
##### Private Repository Example

In this example we'll save the `helloworld-demo-node` image from Docker onto our private registry.

1. clone an example git hub repo. This is an example Node.js code repo 
    ```git clone https://github.com/dockersamples/helloworld-demo-node```

2. go into the new directory

3. Build the docker image
    ```
    docker build -t docker-quickstart .
    ```
  

4. Check to see if the image exists locally
   ```
    mav@sgvmcloud:~/docker_learning/helloworld-demo-node$ docker images
    REPOSITORY          TAG       IMAGE ID       CREATED             SIZE
    docker-quickstart   latest    72432157603b   About an hour ago   142MB
	```
 1.   Test out running the image (optional) 
    ```docker run -d -p 8080:8080 docker-quickstart ```

 2. Use docker tag to tag the docker image. This labels and versions the image, but also most importantly - pushes the image to the private repository. You have to explicitly set the URL as the tag 
     `docker tag docker-quickstart localhost:5400/docker-quickstart:1.0`

    If you do not set `localhost:5400` as the tag, you’ll get into an error when trying to actually push to the repository
	```
    mav@sgvmcloud:~/docker_learning/helloworld-demo-node$ docker push localhost:5400/docker-quickstart:1.0
    The push refers to repository [localhost:5400/docker-quickstart]
    An image does not exist locally with the tag: localhost:5400/docker-quickstart
	```
5.  Push the image to the repository 
    `docker push localhost:5400/docker-quickstart:1.0`

    This will push the image to the docker repository.
    Issue another curl to see the newly pushed image
	```
    mav@sgvmcloud:~/docker_learning/helloworld-demo-node$ curl http://localhost:5400/v2/_catalog
    {"repositories":["docker-quickstart"]}
	```
6.  There’s now a docker image saved in our local registry. Let’s verify that it works. We’ll delete the locally saved images.
	```
    mav@sgvmcloud:~/docker_learning/helloworld-demo-node$ docker rmi 72432157603b --force
    Untagged: docker-quickstart:1.0
    Untagged: docker-quickstart:latest
    Untagged: localhost:5400/docker-quickstart:1.0
    Untagged: localhost:5400/docker-quickstart@sha256:b79e5a62e3db746b7618e25f3e3f3bd34ad9a13b83f359b2c7a232451c7001b3
    Deleted: sha256:72432157603bdc477981e4deb8e6bca67da5b6f795b16299c95abe3bb10ebea1
    mav@sgvmcloud:~/docker_learning/helloworld-demo-node$ docker images
    REPOSITORY    TAG       IMAGE ID       CREATED        SIZE
    hello-world   latest    1b44b5a3e06a   2 weeks ago    10.1kB
    traefik       v3.4      aa215d9f973d   4 weeks ago    227MB
    mysql         9.3       850100bac3be   4 months ago   859MB
    registry      latest    3c52eedeec80   4 months ago   57.7MB
    phpmyadmin    latest    21c6d797c79c   7 months ago   568MB
	```
    We can see no more images pertaining to `docker-quickstart` exist within our local docker unit.

7. Let’s pull our `docker-quickstart` image again. If we issue another curl, we can see that our private registry still contains our image

	```
    mav@sgvmcloud:~/docker_learning/helloworld-demo-node$ curl http://localhost:5400/v2/_catalog
    {"repositories":["docker-quickstart"]}
	```
    Now to pull the image 
    `docker pull localhost:5400/docker-quickstart:1.0`

    And we can verify that the image exists!
   ```
    mav@sgvmcloud:~/docker_learning/helloworld-demo-node$ docker images
    REPOSITORY                         TAG       IMAGE ID       CREATED        SIZE
    localhost:5400/docker-quickstart   1.0       72432157603b   2 hours ago    142MB
	```
     

We now have a Private Registry that will contain image repositories for development. The images will stay on the repository even if the container is restarted. However if the container is removed `docker container rm -v registry` then you’ll loose that data (and all of the images).

 

Through an internal lab, we can then use the `localhost:5400` endpoint as the server, and hide it behind some type of web server like apache, where we can then forward requests to the `localhost:5400` endpoint so it’s able to be accessible from other systems. It's imperative to set up some sort of proxy rule to only allow internal machines to access the registry resource.
 

 