// https://github.com/wasm-bindgen/wasm-bindgen/blob/main/examples/hello_world/webpack.config.js

const path = require('path');
const HtmlWebpackPlugin = require('html-webpack-plugin');
const webpack = require('webpack');

module.exports = {
    entry: "./index.mjs",
    output: {
        path: path.resolve(__dirname, 'dist', 'hello_world'),
        filename: 'index.js',
    },
    plugins: [
        new HtmlWebpackPlugin({
            template: "index.html"
        }),
    ],
    mode: 'development',
    experiments: {
        asyncWebAssembly: true
   }
};