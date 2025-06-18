import { Schema, model } from 'mongoose';


// Define a Mongoose schema and model for your application
// Adjust the schema fields according to your application's requirements
const exampleSchema = new Schema({
  name: { type: String, required: true },
  description: { type: String, required: true },
  createdAt: { type: Date, default: Date.now },
  updatedAt: { type: Date, default: Date.now },
});

// Middleware to update the updatedAt field before saving

export default model('Example', exampleSchema);