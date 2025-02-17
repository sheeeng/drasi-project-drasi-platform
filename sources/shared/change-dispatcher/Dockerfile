# Copyright 2024 The Drasi Authors.
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#     http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.

FROM rust@sha256:fd45a543ed41160eae2ce9e749e5b3c972625b0778104e8962e9bfb113535301 as builder
# rust:1.74
RUN apt-get update && apt-get install -y protobuf-compiler libcurl4 && apt-get clean

WORKDIR /usr/src
COPY ./infrastructure ./infrastructure

RUN cargo new source/shared/change-dispatcher
WORKDIR /usr/src/source/shared/change-dispatcher
COPY ./sources/shared/change-dispatcher/Cargo.toml . 
RUN cargo fetch
COPY ./sources/shared/change-dispatcher .
RUN cargo install --force --path .

    
FROM  gcr.io/distroless/cc@sha256:3b75fdd33932d16e53a461277becf57c4f815c6cee5f6bc8f52457c095e004c8
# gcr.io/distroless/cc 
ENV RUST_BACKTRACE=1
COPY --from=builder /usr/local/cargo/bin/change-dispatcher /usr/local/bin/change-dispatcher
# RUN apt-get update && apt install -y openssl
CMD ["change-dispatcher"]