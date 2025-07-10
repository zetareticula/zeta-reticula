"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.getUsage = exports.addUsage = void 0;
const db_1 = require("../db");
const addUsage = async (usage) => {
    await (0, db_1.query)(`INSERT INTO usage (user_id, tokens_used, cost, timestamp) VALUES ($1, $2, $3, $4)`, [usage.user_id, usage.tokens_used, usage.cost, usage.timestamp]);
};
exports.addUsage = addUsage;
const getUsage = async (userId) => {
    return (await (0, db_1.query)('SELECT * FROM usage WHERE user_id = $1 ORDER BY timestamp DESC', [userId]));
};
exports.getUsage = getUsage;
