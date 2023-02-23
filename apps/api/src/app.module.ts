import { Module } from "@nestjs/common";
import { AppService } from "./app.service";
import { AppController } from "./app.controller";
import { PrismaService } from "./prisma/prisma.service";

@Module({
  controllers: [AppController],
  providers: [AppService, PrismaService]
})
export class AppModule {}
