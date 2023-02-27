import { Task } from "@prisma/client";

export class TaskEntity implements Task {
  readonly id!: number;
  title!: string;
  isDone!: boolean;
  createdAt!: Date;
}
