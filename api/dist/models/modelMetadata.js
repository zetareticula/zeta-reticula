"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.getModel = exports.addModel = void 0;
const db_1 = require("../db");
const addModel = async (metadata) => {
    await (0, db_1.query)(`INSERT INTO models (model_id, user_id, file_name, file_size, format, quantized_path, upload_timestamp, cost)
     VALUES ($1, $2, $3, $4, $5, $6, $7, $8)`, [
        metadata.model_id,
        metadata.user_id,
        metadata.file_name,
        metadata.file_size,
        metadata.format,
        metadata.quantized_path,
        metadata.upload_timestamp,
        metadata.cost,
    ]);
};
exports.addModel = addModel;
const getModel = async (modelId) => {
    const rows = await (0, db_1.query)('SELECT * FROM models WHERE model_id = $1', [modelId]);
    return rows[0];
};
exports.getModel = getModel;
