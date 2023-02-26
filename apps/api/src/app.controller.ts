import { AppService } from "./app.service";
import { CreateTaskDto } from "./task/dto/create-task.dto";
import { UpdateTaskDto } from "./task/dto/update-task.dto";
import { Task } from "./task/entities/task.entity";
import { Controller, Get, Post, Body, Patch, Param, Delete, Put } from "@nestjs/common";
import { ApiOkResponse } from "@nestjs/swagger";

@Controller()
export class AppController {
  constructor(private readonly appService: AppService) {}

  @Post()
  @ApiOkResponse({ type: Task })
  async createTask(@Body() createTaskDto: CreateTaskDto) {
    return await this.appService.createTask(createTaskDto);
  }

  @Get()
  async getTaskList(): Promise<Task[]> {
    return await this.appService.getTaskList();
  }

  @Put(":id")
  @ApiOkResponse({ type: Task })
  async updateTask(@Param("id") id: string, @Body() updateTaskDto: UpdateTaskDto) {
    return await this.appService.updateTask({
      where: { id: Number(id) },
      data: updateTaskDto
    });
  }

  @Delete()
  async deleteAllTask() {
    return await this.appService.deleteAllTask();
  }

  @Delete(":id")
  @ApiOkResponse({ type: Task })
  async deleteTask(@Param("id") id: string) {
    return await this.appService.removeTask({ id: Number(id) });
  }

  // @get(":id")
  // findone(@param("id") id: string) {
  //   return this.appservice.findone(+id);
  // }
  //
  // @patch(":id")
  // update(@param("id") id: string, @body() updateappdto: updateappdto) {
  //   return this.appservice.update(+id, updateappdto);
  // }
}
