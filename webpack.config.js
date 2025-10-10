// https://github.com/wasm-bindgen/wasm-bindgen/blob/main/examples/hello_world/webpack.config.js

const path = require('path');
const HtmlWebpackPlugin = require('html-webpack-plugin');
const webpack = require('webpack');
const WasmPackPlugin = require("@wasm-tool/wasm-pack-plugin");

module.exports = {
    entry: './index.js',
    output: {
        path: path.resolve(__dirname, '..', 'dist', 'hello_world'),
        filename: 'index.js',
    },
    plugins: [
        new HtmlWebpackPlugin(),
        new WasmPackPlugin({
            crateDirectory: __dirname
        }),
    ],
    mode: 'development',
    experiments: {
        asyncWebAssembly: true
   }
};