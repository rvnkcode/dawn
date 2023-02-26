import { PrismaService } from "./prisma/prisma.service";
import { Injectable } from "@nestjs/common";
import { Task, Prisma } from "@prisma/client";

@Injectable()
export class AppService {
  constructor(private prisma: PrismaService) {}

  async createTask(data: Prisma.TaskCreateInput): Promise<Task> {
    return await this.prisma.task.create({ data });
  }

  async getTaskList(): Promise<Task[]> {
    return await this.prisma.task.findMany();
  }

  async updateTask(params: { where: Prisma.TaskWhereUniqueInput; data: Prisma.TaskUpdateInput }): Promise<Task> {
    const { where, data } = params;
    return await this.prisma.task.update({
      where,
      data
    });
  }

  async deleteAllTask() {
    return await this.prisma.task.deleteMany({});
  }

  async removeTask(where: Prisma.TaskWhereUniqueInput) {
    return await this.prisma.task.delete({
      where
    });
  }

  // findOne(id: number) {
  //   return `This action returns a #${id} app`;
  // }

  // remove(id: number) {
  //   return `This action removes a #${id} app`;
  // }
}
