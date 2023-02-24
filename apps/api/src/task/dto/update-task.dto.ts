import { CreateTaskDto } from "./create-task.dto";
import { PartialType } from "@nestjs/mapped-types";
import { IsBoolean } from "class-validator";

export class UpdateTaskDto extends PartialType(CreateTaskDto) {
  title?: string;

  @IsBoolean()
  isDone?: boolean;
}
