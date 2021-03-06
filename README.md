# sugarfunge-k8s

Manage your SugarFunge Infrastructure in **[Kubernetes](https://kubernetes.io)**.

### Kubernetes version support

| Kubernetes Version          | sugarfunge-k8s version                  |
| --------------------------- | --------------------------------------- |
| 1.23                        | 0.1.0 (Latest)                          |

Check the `Cargo.toml` file if you're running from `main` after the first release gets published:
```rust
// feature 1_23 = Kubernetes 1.23
k8s-openapi = { version = "0.15.0", features = ["v1_23"] }`
```

## Usage

* Check the available commands and arguments.

```bash
sugarfunge-k8s --help
```

* Run the infrastructure with the default configuration.

```bash
kubectl create ns test
sugarfunge-k8s create ipfs
sugarfunge-k8s create ipfs -n test
sugarfunge-k8s create node
sugarfunge-k8s create node --config=config.ron -n test
sugarfunge-k8s create explorer
sugarfunge-k8s create status
sugarfunge-k8s create api
# Check the Keycloak section on how to run it with the default configution.
sugarfunge-k8s create keycloak
```

## Build from Source

### Software Requirement

* **[Rust](https://rustup.rs)**: Stable

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
* **[Helm](https://helm.sh/docs/intro/install)**: Stable

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
