# sugarfunge-k8s

Manage your SugarFunge Infrastructure in Kubernetes.

## Usage

```bash
sugarfunge-k8s --help
```

* An optional configuration file can be provided with the `--config` argument following the file path like `sugarfunge-k8s create node --config config.ron`. It should be a [ron](https://github.com/ron-rs/ron) file. An example file is provided in the repository `config.ron` with the default configuration used when not file is provided.

## Build from Source

### Software Requirements

Click the name of each software to go to the setup webpage.

* **[Rust](https://rustup.rs)**: Stable
* **[Docker](https://docs.docker.com/get-docker)**: 20.10.x
* *[Minikube](https://minikube.sigs.k8s.io/docs/start)*: 1.25.x (Tested with Kubernetes v1.23.3) (Optional. You can use your own Kubernetes cluster that matches the version used in the `k8s-openapi` crate in the `Cargo.toml` file)

### Download and Compile

```bash
$ git clone https://github.com/SugarFunge/sugarfunge-k8s.git
$ cd sugarfunge-k8s
$ cargo build --release
# Run
$ ./target/release/sugarfunge-k8s --help
# or
$ cargo run --release -- --help
```

## Keycloak

> This configuration is intended for local and/or testing purposes.

### Additional Dependency
* **[Helm](https://helm.sh/docs/intro/install)**: 3.9.x

### Setup

* Use helm to add the bitnami repo and install the postgres chart with the configuration file provided in the repository.
```
$ helm repo add bitnami https://charts.bitnami.com/bitnami
$ helm install sf-db bitnami/postgresql -f postgres_local.yaml
```

* Create the keycloak service into the Kubernetes cluster with the default configuration.
```
$ sugarfunge-k8s create keycloak
```

## Ingress

> This feature is intended for testing purposes only.

### Additional Dependencies

* **[nginx-ingress-controller](https://kubernetes.github.io/ingress-nginx/deploy)**
* **[cert-manager](https://cert-manager.io/docs/installation)**

### Setup

1. Install the dependencies via `kubectl` or `Helm`.
2. Create a `ClusterIssuer` with `kubectl`. An example is provided below using the staging enviroment of `Let's Encrypt`.
```yaml
apiVersion: cert-manager.io/v1
kind: ClusterIssuer
metadata:
  name: letsencrypt-staging
spec:
  acme:
    server: https://acme-staging-v02.api.letsencrypt.org/directory
    email: youremail@domain.com
    privateKeySecretRef:
      name: letsencrypt-staging
    solvers:
    - http01:
        ingress:
          class: nginx
```

3. Be sure that your `config.ron` includes the name of the `ClusterIssuer`.
```rust
    ingress: Some(
        IngressConfig (
            ...
            tls_issuer: "letsencrypt-staging",
        )
    ),
```

4. The `host` in the `config.ron` for the `IngressConfig` requires to set up subdomains. If the host is `demo.sugarfunge.dev`, it's expected to create all the subdomains that matches each service `name` in the `config.ron` file and points to the nginx service IP address.

```
api.demo.sugarfunge.dev
explorer.demo.sugarfunge.dev
ipfs.demo.sugarfunge.dev
auth.demo.sugarfunge.dev
node.demo.sugarfunge.dev
status.demo.sugarfunge.dev
```

For example, for the `keycloak` service, the `config.ron` the name field for `KeycloakConfig` should be `auth`:
```rust
    keycloak: Some(
        KeycloakConfig (
            name: "auth",
            ...
        )
    ),
```

5. Create the service with the cli tool and wait for the issuer to create and validate the `Let's Encrypt` certificate.
```bash
sugarfunge-k8s create ingress --config config.ron
```
