# To support AppImage on older systems we should build on relatively old distribution,
# see: https://tauri.app/v1/guides/building/linux/#limitations
# Debian Buster LTS support is planned until June 30, 2024.
FROM rust:buster

# Tauri dependencies
RUN apt-get update && apt-get install -y \
        libwebkit2gtk-4.0-dev \
        build-essential \
        curl \
        wget \
        libssl-dev \
        libgtk-3-dev \
        libayatana-appindicator3-dev \
        librsvg2-dev \
        && rm -rf /var/lib/apt/lists/*

# Vue dependencies
RUN apt-get update && \
        curl -fsSL https://deb.nodesource.com/setup_18.x | bash - && \
        apt-get install -y --no-install-recommends nodejs build-essential && \
        rm -rf /var/lib/apt/lists/*

WORKDIR /flasher

COPY ./babel.config.js    ./babel.config.js
COPY ./.browserslistrc    ./.browserslistrc
COPY ./.eslintrc.js       ./.eslintrc.js
COPY ./inscribe-theme     ./inscribe-theme
COPY ./package.json       ./package.json
COPY ./package-lock.json  ./package-lock.json
COPY ./postcss.config.js  ./postcss.config.js
COPY ./public             ./public
COPY ./src                ./src
COPY ./src-tauri          ./src-tauri
COPY ./tailwind.config.js ./tailwind.config.js
COPY ./tsconfig.json      ./tsconfig.json
COPY ./vue.config.js      ./vue.config.js

RUN npm ci

CMD npm run tauri:build
