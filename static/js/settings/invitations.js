class Invitations {
    constructor() {
      this.add_collaborators = $(".create-new-token-btn, .create-new-token-mobile-btn, .add_collaborators_btn");
      this.collaborators_modal = $('#new_collaborator_modal');
      
      this.owner_btn = $('.ownership_btn, .transfer');
      this.owner_modal = $('#transfer_owner_modal');

      this.newTokenInput = this.collaborators_modal.find('input')
      this.newTokenItemTemplate = $('.token-item-template .token-item')
      this.tokensList = $('.tokens-list')
      this.inputEmail = $('.add_collaborators_form #submit-btn')
      this.init()
    }
  
    init() {
      $(document).foundation()
      $('.token-created-at').timeago();
  
      this.add_collaborators.click(() => {
        // $('#new_collaborator_modal').foundation('open');
        this.collaborators_modal.foundation('open');
        this.newTokenInput.focus();
      })

      this.collaborators_modal.find('.submit').on('click', () => {
        this.collaborators_modal.foundation('close')
        //call function
        //this.submitNewToken()
      })

      this.collaborators_modal.find('.cancel').on('click', () => {
        this.collaborators_modal.foundation('close')
      })
      
      this.owner_btn.click(() => {
        $('#transfer_owner_modal').foundation('open');
        this.newTokenInput.focus();
      })

      this.owner_modal.find('.submit').on('click', () => {
        this.owner_modal.foundation('close')
        //call function
        //this.submitNewToken()
      })

      this.owner_modal.find('.cancel').on('click', () => {
        this.owner_modal.foundation('close')
      })
      



      this.newTokenInput.on('keypress', (e) => {
        if (e.key == "Enter") {
          this.collaborators_modal.foundation('close')
          this.submitNewToken()
          this.newTokenInput.val('')
        }
      })

      // handle required input
      $('#user_email').change(() => {
        $('.add_collaborators_btn').css('background-color','var(--blue-color)');
      }) 
 
    }
  
    submitNewToken1() {
      const tokenName = this.newTokenInput.val();
      if (!tokenName) return
      $.ajax({
        type: 'PUT',
        dataType: "json",
        url: '/api/v1/tokens',
        contentType: "application/json",
        processData: false,
        headers: {},
        data: JSON.stringify({"name": tokenName}),
        success: (data, status, xhr) => {
          $('.no-tokens').remove()
          const newTokenItem = this.newTokenItemTemplate.clone()
          newTokenItem.data('id', data.id)
          newTokenItem.find('.token-name').text(data.name)
          newTokenItem.find('.token-plaintext').text(data.token)
  
          this.tokensList.append(newTokenItem)
          return data
        },
        error: function (xhr, status, errorThrown) {
          $(".tokens-error").text(xhr.responseText)
          return errorThrown
        },
      })
    }
  }
  