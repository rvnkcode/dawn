import { CreateTaskDto } from "../../Api";
import { api, listAtom } from "../selectors/task";
import { PlusOutlined } from "@ant-design/icons";
import { Button, Form, Input, message } from "antd";
import { useRecoilState } from "recoil";

export default function TaskInputForm() {
  const [form] = Form.useForm();
  const [list, setList] = useRecoilState(listAtom);

  // POST
  const createTask = (value: CreateTaskDto) => {
    api.task
      .appControllerCreateTask(value)
      .then((response) => {
        setList(list.concat(response.data));
        form.resetFields();
      })
      .catch((error) => {
        console.error(error);
        message.error(`${error}`);
      });
  };

  return (
    <Form onFinish={createTask} form={form}>
      <Input.Group compact>
        <Form.Item name="title" noStyle rules={[{ required: true }]}>
          <Input style={{ width: "calc(100% - 46px)" }} autoFocus placeholder="New To-Do" required autoComplete="off" />
        </Form.Item>
        <Button type="primary" htmlType="submit">
          <PlusOutlined />
        </Button>
      </Input.Group>
    </Form>
  );
}
