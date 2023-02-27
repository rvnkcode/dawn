import { AppController } from "./app.controller";
import { AppService } from "./app.service";
import { PrismaService } from "./prisma/prisma.service";
import { Module } from "@nestjs/common";

@Module({
  controllers: [AppController],
  providers: [AppService, PrismaService]
})
export class AppModule {}
