import { AppModule } from "./app.module";
import { ValidationPipe } from "@nestjs/common";
import { NestFactory } from "@nestjs/core";
import { DocumentBuilder, SwaggerModule } from "@nestjs/swagger";
import { existsSync, writeFileSync } from "fs";
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
  writeFileSync("./swagger.yaml", dump(document, {}));
  // Export swagger.yaml for devcontainer
  if (existsSync(`/memo`)) {
    writeFileSync(`/memo/swagger.yaml`, dump(document, {}));
  }

  await app.listen(3000);
}
bootstrap();
