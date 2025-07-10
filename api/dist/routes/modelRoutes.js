"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.handler = void 0;
const express_1 = require("express");
const modelController_1 = require("../controllers/modelController");
const authMiddleware_1 = require("../middleware/authMiddleware");
const uploadMiddleware_1 = require("../middleware/uploadMiddleware");
const handler = async (req, res) => {
    const router = (0, express_1.Router)();
    router.post('/', authMiddleware_1.authenticate, uploadMiddleware_1.uploadModelMiddleware, uploadMiddleware_1.validateUpload, modelController_1.postModel);
    router.get('/:modelId', authMiddleware_1.authenticate, modelController_1.getModelStatus);
    return new Promise((resolve) => {
        router(req, res, () => resolve());
    });
};
exports.handler = handler;
exports.default = exports.handler;
