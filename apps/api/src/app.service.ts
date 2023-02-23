import { Injectable } from "@nestjs/common";
import { PrismaService } from "./prisma/prisma.service";
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

  async deleteAllTask() {
    return await this.prisma.task.deleteMany({});
  }

  // findOne(id: number) {
  //   return `This action returns a #${id} app`;
  // }

  // update(id: number, updateAppDto: UpdateAppDto) {
  //   return `This action updates a #${id} app`;
  // }

  // remove(id: number) {
  //   return `This action removes a #${id} app`;
  // }
}
