import { CreateTaskDto } from "./create-task.dto";
import { PartialType } from "@nestjs/mapped-types";
import { IsBoolean, IsOptional } from "class-validator";

export class UpdateTaskDto extends PartialType(CreateTaskDto) {
  title?: string;

  @IsOptional()
  @IsBoolean()
  isDone?: boolean;
}
