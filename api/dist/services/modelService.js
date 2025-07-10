"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.uploadModel = void 0;
const fileUtils_1 = require("../utils/fileUtils");
const billing_1 = require("../utils/billing");
const modelMetadata_1 = require("../models/modelMetadata");
const userUsage_1 = require("../models/userUsage");
const uuid_1 = require("uuid");
const uploadModel = async (file, userId) => {
    const error = (0, fileUtils_1.validateModelFile)(file);
    if (error)
        throw new Error(error);
    const fileUrl = await (0, fileUtils_1.saveFile)(file);
    const fileSize = file.size;
    const cost = (0, billing_1.calculateUploadCost)(fileSize);
    const modelId = (0, uuid_1.v4)();
    const metadata = {
        model_id: modelId,
        user_id: userId,
        file_name: file.originalname,
        file_size: fileSize,
        format: path.extname(file.originalname).toLowerCase(),
        quantized_path: `${fileUrl}.rkv`, // Simulate quantization
        upload_timestamp: new Date().toISOString(),
        cost,
    };
    console.log(`Quantizing ${file.originalname} to ${metadata.quantized_path}`);
    await (0, modelMetadata_1.addModel)(metadata);
    await (0, userUsage_1.addUsage)({
        user_id: userId,
        tokens_used: 0,
        cost,
        timestamp: metadata.upload_timestamp,
    });
    return { modelId, cost };
};
exports.uploadModel = uploadModel;
