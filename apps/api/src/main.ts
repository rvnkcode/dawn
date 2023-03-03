import { AppModule } from "./app.module";
import { ValidationPipe } from "@nestjs/common";
import { NestFactory } from "@nestjs/core";
import { DocumentBuilder, SwaggerModule } from "@nestjs/swagger";
import { writeFileSync } from "fs";
import { dump } from "js-yaml";

async function bootstrap() {
  const app = await NestFactory.create(AppModule);
  app.enableCors();
  // DTO validation enables
  app.useGlobalPipes(new ValidationPipe());

  // Generate http://localhost:3000/api
  const config = new DocumentBuilder()
    .setTitle(`GTD API`)
    .setDescription(`Getting Things Done API description`)
    .setVersion(`0.0.1`)
    .addTag(`Task`)
    .build();
  const document = SwaggerModule.createDocument(app, config);
  SwaggerModule.setup(`api`, app, document);

  // Export swagger.yaml
  try {
    writeFileSync("./swagger.yaml", dump(document, {}));
    console.log(`swagger.yaml exported to project directory`);
  } catch (error) {
    console.error(`swagger.yaml could not export to project directory`, error);
  }

  await app.listen(3000);
}
bootstrap();
