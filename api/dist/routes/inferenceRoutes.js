"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.handler = void 0;
const express_1 = require("express");
const inferenceController_1 = require("../controllers/inferenceController");
const authMiddleware_1 = require("../middleware/authMiddleware");
// Export as a Vercel serverless function
const handler = async (req, res) => {
    const router = (0, express_1.Router)();
    router.post('/', authMiddleware_1.authenticate, inferenceController_1.postInference);
    router.get('/usage', authMiddleware_1.authenticate, inferenceController_1.getUsageHistory);
    // Vercel expects a handler function
    return new Promise((resolve) => {
        router(req, res, () => resolve());
    });
};
exports.handler = handler;
exports.default = exports.handler;
