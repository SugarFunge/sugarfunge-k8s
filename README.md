# sugarfunge-k8s

```
sugarfunge-k8s 0.1.0
Manage your SugarFunge Infrastructure in Kubernetes

USAGE:
    sugarfunge-k8s [OPTIONS] <ACTION> <SERVICE>

ARGS:
    <ACTION>     [possible values: create, delete]
    <SERVICE>    Name of the service [possible values: api, explorer, keycloak, node, status]

OPTIONS:
    -c <CONFIG>                    
    -h, --help                     Print help information
    -n, --namespace <NAMESPACE>    [default: default]
    -V, --version                  Print version information
```

## Software Requirements

Click the name of each software to go to the setup webpage.

* [Rust](https://rustup.rs): Stable
* [Docker](https://docs.docker.com/get-docker): 20.10.x
* [Minikube](https://minikube.sigs.k8s.io/docs/start): 1.25.x (Tested with Kubernetes v1.23.3)
* [Helm](https://helm.sh/docs/intro/install): 3.9.x (Optional. Only required for running Keycloak locally)

## Setup

### Installing from source

* Clone the repo and change directory into it.
```
$ git clone https://github.com/SugarFunge/sugarfunge-k8s.git
$ cd sugarfunge-k8s
```

* Check the help command for the options available `cargo run -- --help`. It will compile the cli application if you haven't ran it before.

* An optional configuration file can be provided with the `-c` argument following the file path like `cargo run -- create node -c config.ron`. It should be a `ron` file. An example file is provided in the repository `config.ron` with the default configuration used when not file is provided.

#### Additional Keycloak configuration to run with a local Postgres instance

* Use helm to add the bitnami repo and install the postgres chart with the configuration file provided in the repository.
```
$ helm repo add bitnami https://charts.bitnami.com/bitnami
$ helm install sf-db bitnami/postgresql -f postgres_local.yaml
```

* Create the keycloak service into the Kubernetes cluster with the default configuration.
```
$ cargo run -- create keycloak
```
