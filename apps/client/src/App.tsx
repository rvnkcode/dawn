import FooterComponent from "./Components/FooterComponent";
import ListComponent from "./Components/ListComponent";
import PageTitleComponent from "./Components/PageTitleComponent";
import TaskEditModal from "./Components/TaskEditModal";
import TaskInputForm from "./Components/TaskInputForm";
import "./style.css";
import Layout from "antd/es/layout/layout";
import styled from "styled-components";

// CSS
const Header = styled.header`
  position: sticky;
  top: 0;
  z-index: 1;
  width: 95%;
  background: white;
  padding-top: 1rem;
  margin-bottom: 1.5rem;
`;

function App() {
  return (
    <Layout
      style={{
        background: "none",
        margin: "auto",
        width: "95%",
        maxWidth: "960px"
      }}
    >
      <Header style={{ width: "100%" }}>
        <PageTitleComponent />
        <TaskInputForm />
      </Header>
      <ListComponent />
      <TaskEditModal />
      <FooterComponent />
    </Layout>
  );
}

export default App;
