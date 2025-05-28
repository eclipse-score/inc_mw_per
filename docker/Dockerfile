# Ubuntu 24.04
FROM ubuntu@sha256:dc17125eaac86538c57da886e494a34489122fb6a3ebb6411153d742594c2ddc

RUN apt-get update && \
    apt-get install --no-install-recommends -y \
    ca-certificates curl file build-essential \
    autoconf automake autotools-dev libtool xutils-dev \
    less vim

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain nightly
ENV PATH="$PATH:/root/.cargo/bin"
RUN rustup component add rust-src --toolchain nightly-x86_64-unknown-linux-gnu

COPY ./qnx-setup-2.0.4-202501021438-linux.run .
RUN chmod +x qnx-setup-2.0.4-202501021438-linux.run
RUN ./qnx-setup-2.0.4-202501021438-linux.run force-override disable-auto-start agree-to-license-terms /root/qnx

ARG MYQNX_LICENSE
ARG MYQNX_USER
ARG MYQNX_PASSWORD

RUN /root/qnx/qnxsoftwarecenter/qnxsoftwarecenter_clt -activateLicenseKey ${MYQNX_LICENSE}
RUN /root/qnx/qnxsoftwarecenter/qnxsoftwarecenter_clt -mirrorBaseline qnx800 -myqnx.user="${MYQNX_USER}" -myqnx.password="${MYQNX_PASSWORD}"
RUN /root/qnx/qnxsoftwarecenter/qnxsoftwarecenter_clt -cleanInstall -destination ~/qnx800 -installBaseline com.qnx.qnx800 -myqnx.user="${MYQNX_USER}" -myqnx.password="${MYQNX_PASSWORD}"

COPY init.sh /
RUN chmod +x /init.sh

RUN mkdir /data
WORKDIR /data

ENTRYPOINT [ "/bin/bash", "/init.sh", "bash" ]
