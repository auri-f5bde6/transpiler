import path from "node:path";
import {fileURLToPath} from "node:url";
import HtmlWebpackPlugin from "html-webpack-plugin";
import webpack from "webpack";
import WasmPackPlugin from "@wasm-tool/wasm-pack-plugin";

const __filename = fileURLToPath(import.meta.url)
const __dirname = path.dirname(__filename)

const config: webpack.Configuration = {
    entry: './web/index.ts',
    output: {
        path: path.resolve(__dirname, '..', 'docs'),
        filename: 'web/index.js',
    },
    plugins: [
        new HtmlWebpackPlugin({
            template: 'web/index.html'
        }),
        new WasmPackPlugin({
            crateDirectory: __dirname
        }),
    ],
    mode: 'development',
    experiments: {
        asyncWebAssembly: true,
        css: true,
    },
    module: {
        parser: {
            "css/auto": {
                exportType: "style",
            },
        },
    },

};
export default config;
