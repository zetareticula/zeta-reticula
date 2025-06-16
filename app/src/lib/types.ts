export interface User {
    id: number;
    name: string;
    email: string;
  }
  
  export interface Feedback {
    id: number;
    userId: number;
    message: string;
    createdAt: Date;
  }