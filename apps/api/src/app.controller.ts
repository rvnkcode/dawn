import { Controller, Get, Post, Body, Patch, Param, Delete } from "@nestjs/common";
import { ApiOkResponse } from "@nestjs/swagger";
import { AppService } from "./app.service";
import { CreateTaskDto } from "./dto/create-task.dto";
import { Task } from "./entities/task.entity";
// import { UpdateAppDto } from "./dto/update-task.dto";

@Controller()
export class AppController {
  constructor(private readonly appService: AppService) {}

  @Post()
  @ApiOkResponse({type: Task})
  async createTask(@Body() createTaskDto: CreateTaskDto) {
    return await this.appService.createTask(createTaskDto);
  }

  @Get()
  async getTaskList(): Promise<Task[]> {
    return await this.appService.getTaskList();
  }

  @Delete()
  async deleteAllTask() {
    return await this.appService.deleteAllTask();
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
